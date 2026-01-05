import { listen } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';

const el = document.getElementById('app');

el.innerHTML = `
  <div style="font-family: system-ui, sans-serif; padding: 16px; max-width: 920px;">
    <h2>SLLV Desktop</h2>
    <p>Encode any file/folder into frames, or decode frames into a recovered <code>.tar</code> archive.</p>

    <div style="display:flex; gap:12px; flex-wrap: wrap;">
      <div style="flex: 1; min-width: 360px; padding: 12px; border: 1px solid #ccc; border-radius: 10px;">
        <h3>Encode</h3>

        <div>
          <button id="pick_in">Pick input (file/folder)</button>
          <div id="enc_in" style="margin-top:6px; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: 12px; word-break: break-all;"></div>
        </div>

        <div style="height:10px"></div>

        <div>
          <button id="pick_out">Pick output frames folder</button>
          <div id="enc_out" style="margin-top:6px; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: 12px; word-break: break-all;"></div>
        </div>

        <div style="height:10px"></div>
        <button id="enc_btn">Encode</button>
      </div>

      <div style="flex: 1; min-width: 360px; padding: 12px; border: 1px solid #ccc; border-radius: 10px;">
        <h3>Decode</h3>

        <div>
          <button id="pick_frames">Pick frames folder</button>
          <div id="dec_in" style="margin-top:6px; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: 12px; word-break: break-all;"></div>
        </div>

        <div style="height:10px"></div>

        <div>
          <button id="pick_tar">Pick output .tar path</button>
          <div id="dec_out" style="margin-top:6px; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: 12px; word-break: break-all;"></div>
        </div>

        <div style="height:10px"></div>
        <button id="dec_btn">Decode</button>
      </div>
    </div>

    <div style="margin-top:14px; padding: 12px; border: 1px solid #ccc; border-radius: 10px;">
      <div style="display:flex; align-items:center; justify-content: space-between; gap: 10px;">
        <div>
          <div style="font-weight: 600;" id="stage">Idle</div>
          <div style="font-size: 12px; opacity: 0.8" id="pct"></div>
        </div>
      </div>
      <progress id="bar" value="0" max="1" style="width:100%; height: 18px; margin-top: 10px;"></progress>
    </div>

    <pre id="log" style="margin-top: 14px; padding: 12px; background: #111; color: #0f0; border-radius: 10px; height: 220px; overflow:auto;"></pre>
  </div>
`;

let encodeInput = '';
let encodeOut = '';
let decodeIn = '';
let decodeOut = '';

const log = (s) => {
  const pre = document.getElementById('log');
  pre.textContent += s + "\n";
  pre.scrollTop = pre.scrollHeight;
}

const setProgress = (stage, done, total) => {
  document.getElementById('stage').textContent = stage;
  const bar = document.getElementById('bar');
  bar.max = total || 1;
  bar.value = done || 0;
  const pct = total ? ((done / total) * 100).toFixed(2) : '';
  document.getElementById('pct').textContent = total ? `${pct}%` : '';
}

await listen('progress', (event) => {
  const p = event.payload;
  setProgress(p.stage, p.done, p.total);
});

await listen('task_result', (event) => {
  const r = event.payload;
  log((r.ok ? '[ok] ' : '[err] ') + r.message);
});

// Pickers

document.getElementById('pick_in').onclick = async () => {
  const v = await open({ multiple: false, directory: false });
  if (v) {
    encodeInput = v;
    document.getElementById('enc_in').textContent = v;
  }
};

document.getElementById('pick_out').onclick = async () => {
  const v = await open({ multiple: false, directory: true });
  if (v) {
    encodeOut = v;
    document.getElementById('enc_out').textContent = v;
  }
};

document.getElementById('pick_frames').onclick = async () => {
  const v = await open({ multiple: false, directory: true });
  if (v) {
    decodeIn = v;
    document.getElementById('dec_in').textContent = v;
  }
};

document.getElementById('pick_tar').onclick = async () => {
  // dialog plugin doesn't provide save dialog here; use a pragmatic input for now.
  // Next increment will add a proper save dialog.
  const v = prompt('Enter output tar path (e.g., /home/user/Downloads/recovered.tar)');
  if (v) {
    decodeOut = v;
    document.getElementById('dec_out').textContent = v;
  }
};

// Actions

document.getElementById('enc_btn').onclick = async () => {
  if (!encodeInput || !encodeOut) {
    log('[encode] pick input + output folder first');
    return;
  }
  log(`[encode] input=${encodeInput}`);
  try {
    await invoke('encode_path', { input: encodeInput, outDir: encodeOut });
    log('[encode] started');
  } catch (e) {
    log('[encode] error: ' + e);
  }
};

document.getElementById('dec_btn').onclick = async () => {
  if (!decodeIn || !decodeOut) {
    log('[decode] pick frames folder + output path first');
    return;
  }
  log(`[decode] in_dir=${decodeIn}`);
  try {
    await invoke('decode_frames', { inDir: decodeIn, output: decodeOut });
    log('[decode] started');
  } catch (e) {
    log('[decode] error: ' + e);
  }
};
