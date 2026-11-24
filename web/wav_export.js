// Pure client-side stereo 16-bit PCM RIFF/WAVE encoder
export function encodeWav16(left, right, sampleRate) {
  const n = Math.min(left.length, right.length);
  const dataSize = n * 4;
  const buf = new ArrayBuffer(44 + dataSize);
  const v = new DataView(buf);
  const str = (pos, s) => {
    for (let i = 0; i < s.length; i++) v.setUint8(pos + i, s.charCodeAt(i));
  };
  str(0, 'RIFF'); v.setUint32(4, 36 + dataSize, true); str(8, 'WAVE');
  str(12, 'fmt '); v.setUint32(16, 16, true); v.setUint16(20, 1, true);
  v.setUint16(22, 2, true); v.setUint32(24, sampleRate, true);
  v.setUint32(28, sampleRate * 4, true); v.setUint16(32, 4, true);
  v.setUint16(34, 16, true); str(36, 'data'); v.setUint32(40, dataSize, true);

  let p = 44;
  for (let i = 0; i < n; i++) {
    let l = Math.max(-1, Math.min(1, left[i]));
    let r = Math.max(-1, Math.min(1, right[i]));
    v.setInt16(p, (l < 0 ? l * 32768 : l * 32767) | 0, true);
    v.setInt16(p + 2, (r < 0 ? r * 32768 : r * 32767) | 0, true);
    p += 4;
  }
  return new Blob([buf], { type: 'audio/wav' });
}

// Triggers browser download for a generated audio Blob
export function downloadWav(blob, filename = 'resonx_recording.wav') {
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.style.display = 'none';
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  setTimeout(() => {
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }, 1000);
}

