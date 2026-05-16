//! Integration tests for sllv-core.
//!
//! These tests exercise the full encode → decode pipeline so regressions in
//! bit-packing, frame ordering, FEC, or palette classification are caught
//! before they reach users.

#[cfg(test)]
mod round_trip {
    use crate::{
        raster::{decode_frames_dir_to_bytes_with_params, encode_bytes_to_frames_dir, RasterParams},
    };

    /// Helper: write `data` to a temp dir of frames, then decode it back and
    /// assert byte-for-byte equality.
    fn round_trip(data: &[u8], params: RasterParams) {
        let tmp = tempfile::tempdir().expect("tempdir");
        let out_dir = tmp.path().join("frames");

        encode_bytes_to_frames_dir(data, "test.bin", &out_dir, &params)
            .expect("encode failed");

        let recovered = decode_frames_dir_to_bytes_with_params(&out_dir, &params)
            .expect("decode failed");

        assert_eq!(
            data, recovered.as_slice(),
            "round-trip mismatch: input len={}, output len={}",
            data.len(), recovered.len()
        );
    }

    // ── No-FEC path ───────────────────────────────────────────────────────────

    #[test]
    fn round_trip_empty_no_fec() {
        let p = RasterParams { fec: None, ..RasterParams::default() };
        round_trip(&[], p);
    }

    #[test]
    fn round_trip_single_byte_no_fec() {
        let p = RasterParams { fec: None, ..RasterParams::default() };
        round_trip(&[0xAB], p);
    }

    #[test]
    fn round_trip_small_no_fec() {
        let p = RasterParams { fec: None, deskew: false, ..RasterParams::default() };
        let data: Vec<u8> = (0u8..=255).collect();
        round_trip(&data, p);
    }

    #[test]
    fn round_trip_multi_frame_no_fec() {
        // Forces multiple frames by using a small chunk size
        let p = RasterParams {
            fec: None,
            deskew: false,
            chunk_bytes: 512,
            ..RasterParams::default()
        };
        let data: Vec<u8> = (0u8..=255).cycle().take(4096).collect();
        round_trip(&data, p);
    }

    #[test]
    fn round_trip_all_zero_bytes_no_fec() {
        let p = RasterParams { fec: None, deskew: false, ..RasterParams::default() };
        let data = vec![0u8; 1024];
        round_trip(&data, p);
    }

    #[test]
    fn round_trip_all_ones_bytes_no_fec() {
        let p = RasterParams { fec: None, deskew: false, ..RasterParams::default() };
        let data = vec![0xFFu8; 1024];
        round_trip(&data, p);
    }

    /// Regression test: payloads that legitimately end in 0x00 bytes must be
    /// recovered exactly.  The old decoder stripped trailing nulls, silently
    /// truncating data that happened to end in zeros.
    #[test]
    fn round_trip_trailing_null_bytes() {
        let p = RasterParams { fec: None, deskew: false, ..RasterParams::default() };
        // Several patterns that end in null bytes
        for data in [
            vec![0xDE, 0xAD, 0x00, 0x00, 0x00],
            vec![0xBE, 0xEF, 0x00],
            vec![0x00u8; 64],
            vec![0xFF, 0x00, 0xFF, 0x00],
        ] {
            round_trip(&data, p.clone());
        }
    }

    // ── FEC path ──────────────────────────────────────────────────────────────

    #[test]
    fn round_trip_small_with_fec() {
        use crate::fec::FecParams;
        let p = RasterParams {
            deskew: false,
            fec: Some(FecParams {
                data_shards: 4,
                parity_shards: 2,
                shard_bytes: 128,
            }),
            ..RasterParams::default()
        };
        let data: Vec<u8> = (0u8..=255).collect();
        round_trip(&data, p);
    }

    #[test]
    fn round_trip_larger_with_fec() {
        use crate::fec::FecParams;
        let p = RasterParams {
            deskew: false,
            fec: Some(FecParams {
                data_shards: 10,
                parity_shards: 4,
                shard_bytes: 256,
            }),
            ..RasterParams::default()
        };
        let data: Vec<u8> = (0u8..=255).cycle().take(8192).collect();
        round_trip(&data, p);
    }

    /// FEC loss simulation: drop up to `parity_shards` frame files from the
    /// encoded output, then decode.  This actually exercises the RS
    /// reconstruct() code path — the previous FEC tests only used the happy
    /// path where every shard was present.
    #[test]
    fn fec_loss_simulation() {
        use crate::fec::FecParams;

        let parity = 4usize;
        let p = RasterParams {
            deskew: false,
            fec: Some(FecParams {
                data_shards: 8,
                parity_shards: parity,
                shard_bytes: 256,
            }),
            ..RasterParams::default()
        };

        let data: Vec<u8> = (0u8..=255).cycle().take(4096).collect();
        let tmp = tempfile::tempdir().expect("tempdir");
        let out_dir = tmp.path().join("frames");

        encode_bytes_to_frames_dir(&data, "loss_test.bin", &out_dir, &p)
            .expect("encode failed");

        // Collect all frame files, sort them, and delete the last `parity` of
        // them.  Since frames are named frame_NNNNNN.png, the last N frames
        // correspond to parity shards (by convention they're appended after
        // data shards in the FEC group).  Even if that's not exactly true, the
        // RS decoder should recover as long as we drop no more than `parity`
        // frames total across the whole stream.
        let mut frames: Vec<_> = std::fs::read_dir(&out_dir)
            .expect("read_dir")
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().map(|x| x == "png").unwrap_or(false))
            .collect();
        frames.sort();

        // Drop up to `parity` frames from the tail
        let drop_count = parity.min(frames.len());
        for f in frames.iter().rev().take(drop_count) {
            std::fs::remove_file(f).expect("remove frame");
        }

        let recovered = decode_frames_dir_to_bytes_with_params(&out_dir, &p)
            .expect("decode with losses failed");

        assert_eq!(
            data, recovered.as_slice(),
            "FEC loss simulation mismatch: dropped {} frames", drop_count
        );
    }

    // ── Bit-packing edge cases ─────────────────────────────────────────────────
    //
    // 3-bit symbols pack across byte boundaries. These sizes exercise those
    // boundary conditions explicitly.

    #[test]
    fn round_trip_3bit_boundary_sizes() {
        let p = RasterParams { fec: None, deskew: false, ..RasterParams::default() };
        // 3 bytes = 8 symbols exactly, 5 bytes = 13 symbols (crosses several boundaries)
        for size in [1, 2, 3, 5, 7, 8, 9, 15, 16, 24, 100, 333] {
            let data: Vec<u8> = (0u8..).take(size).collect();
            round_trip(&data, p.clone());
        }
    }

    // ── Palette classifier ────────────────────────────────────────────────────

    #[test]
    fn palette_classifier_all_symbols() {
        use crate::palette::Palette8;
        let pal = Palette8::Basic;
        // Every canonical colour must round-trip through the fast classifier
        for sym in 0u8..8 {
            let c = pal.color(sym).unwrap();
            assert_eq!(
                pal.symbol_from_rgb_nearest(c.r, c.g, c.b), sym,
                "symbol {sym} did not round-trip"
            );
        }
    }
}
