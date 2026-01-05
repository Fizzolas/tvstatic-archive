import { invoke } from '@tauri-apps/api/core';

const el = document.getElementById('app');

el.innerHTML = `
  <div style="font-family: system-ui, sans-serif; padding: 16px; max-width: 860px;">
    <h2>SLLV Desktop (Increment 2a)</h2>
    <p>Encode any file/folder into frames, or decode frames into a recovered <code>.tar</code> archive.</p>

    <div style="display:flex; gap:12px; flex-wrap: wrap;">
      <div style="flex: 1; min-width: 320px; padding: 12px; border: 1px solid #ccc; border-radius: 10px;">
        <h3>Encode</h3>
        <label>Input path (file or folder)</label><br />
        <input id="enc_in" style="width:100%" placeholder="/path/to/anything" />
        <div style="height:8px"></div>
        <label>Output frames dir</label><br />
        <input id="enc_out" style="width:100%" placeholder="/path/to/out_frames" />
        <div style="height:10px"></div>
        <button id="enc_btn">Encode</button>
      </div>

      <div style="flex: 1; min-width: 320px; padding: 12px; border: 1px solid #ccc; border-radius: 10px;">
        <h3>Decode</h3>
        <label>Input frames dir</label><br />
        <input id="dec_in" style="width:100%" placeholder="/path/to/out_frames" />
        <div style="height:8px"></div>
        <label>Output tar path</label><br />
        <input id="dec_out" style="width:100%" placeholder="/path/to/recovered.tar" />
        <div style="height:10px"></div>
        <button id="dec_btn">Decode</button>
      </div>
    </div>

    <pre id="log" style="margin-top: 14px; padding: 12px; background: #111; color: #0f0; border-radius: 10px; height: 220px; overflow:auto;"></pre>
  </div>
`;

const log = (s) => {
  const pre = document.getElementById('log');
  pre.textContent += s + "\n";
  pre.scrollTop = pre.scrollHeight;
}

document.getElementById('enc_btn').onclick = async () => {
  const input = document.getElementById('enc_in').value;
  const out_dir = document.getElementById('enc_out').value;
  log(`[encode] input=${input}`);
  try {
    await invoke('encode_path', { input, outDir: out_dir });
    log('[encode] done');
  } catch (e) {
    log('[encode] error: ' + e);
  }
};

document.getElementById('dec_btn').onclick = async () => {
  const in_dir = document.getElementById('dec_in').value;
  const output = document.getElementById('dec_out').value;
  log(`[decode] in_dir=${in_dir}`);
  try {
    await invoke('decode_frames', { inDir: in_dir, output });
    log('[decode] done');
  } catch (e) {
    log('[decode] error: ' + e);
  }
};
