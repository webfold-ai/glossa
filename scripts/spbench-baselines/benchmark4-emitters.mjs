import * as yaml from 'js-yaml';
import gettextParser from 'gettext-parser';

const N = 500;
const catalog = {};
for (let i = 0; i < N; i++) catalog[`key_${i}`] = `Translated text #${i} for user {user}`;

function timeOp(fn) {
  const t0 = process.hrtime.bigint();
  const out = fn();
  const t1 = process.hrtime.bigint();
  return { ms: Number(t1 - t0) / 1e6, size: out.length };
}

// warm up
for (let i = 0; i < 50; i++) { JSON.stringify(catalog); yaml.dump(catalog); }

const jsonR = timeOp(() => JSON.stringify(catalog));
const yamlR = timeOp(() => yaml.dump(catalog));
const poR = timeOp(() => {
  const translations = { '': {} };
  for (const [k, v] of Object.entries(catalog)) {
    translations[''][k] = { msgid: k, msgstr: [v] };
  }
  return gettextParser.po.compile({ charset: 'utf-8', headers: {}, translations }).toString();
});

const totalMs = jsonR.ms + yamlR.ms + poR.ms; // json+yaml+po, fluent library not installed (skip rather than fake)
const termsPerSec = (N * 3) / (totalMs / 1000);

console.log(JSON.stringify({
  methodology: "real JSON.stringify + js-yaml.dump + gettext-parser PO compile on a 500-key catalog (glossa's own benchmark uses 500 keys x 4 formats; fluent.js omitted here rather than faked, so this is JSON+YAML+PO only, 3 formats not 4)",
  node_version: process.version,
  json_ms: jsonR.ms,
  yaml_ms: yamlR.ms,
  po_ms: poR.ms,
  total_ms_3_formats: totalMs,
  terms_per_sec_3_formats: termsPerSec,
}, null, 2));
