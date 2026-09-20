// Benchmark 2 real incumbent equivalent: merging translation records into a
// key-value store, the Node.js way a Lingo.dev/i18next-style tool would
// (plain object property assignment — the natural "JSON merge" a JS
// toolchain would do, matching glossa's own baseline label "JSON merge, Node.js").
function bench2() {
  const N = 10000; // matches glossa's b2_count = count(5000) * 2
  const latenciesUs = [];
  const store = {};
  const t0 = process.hrtime.bigint();
  for (let i = 0; i < N; i++) {
    const s = process.hrtime.bigint();
    const key = `Term #${i} in application context`;
    store[key] = { es: `Término #${i}`, provider: 'spbench', quality: 1.0, updatedAt: Date.now() };
    const e = process.hrtime.bigint();
    latenciesUs.push(Number(e - s) / 1000);
  }
  const t1 = process.hrtime.bigint();
  const totalMs = Number(t1 - t0) / 1e6;
  latenciesUs.sort((a, b) => a - b);
  return { totalMs, meanUs: (totalMs * 1000) / N, perKeyUs: (totalMs * 1000) / N, n: N };
}

// Benchmark 3 real incumbent equivalent: a regex-based slot/tag preservation
// validator (the naive approach glossa's own label "Regex wrapper" describes)
// checking that {placeholders} and <N>numbered tags</N> in a translation
// match the source, on the exact same sample strings from glossa's source.
function bench3() {
  const validSrc = "Welcome {user}, click <0>here</0> to proceed.";
  const validTrans = "Bienvenido {user}, haz clic <0>aquí</0> para continuar.";
  const slotRe = /\{[^}]+\}/g;
  const tagRe = /<(\d+)>/g;
  function validate(src, trans) {
    const srcSlots = [...src.matchAll(slotRe)].map((m) => m[0]).sort();
    const transSlots = [...trans.matchAll(slotRe)].map((m) => m[0]).sort();
    const srcTags = [...src.matchAll(tagRe)].map((m) => m[1]).sort();
    const transTags = [...trans.matchAll(tagRe)].map((m) => m[1]).sort();
    return JSON.stringify(srcSlots) === JSON.stringify(transSlots) && JSON.stringify(srcTags) === JSON.stringify(transTags);
  }
  const N = 5000; // matches glossa's b3_count = count
  // warm up
  for (let i = 0; i < 500; i++) validate(validSrc, validTrans);
  const t0 = process.hrtime.bigint();
  let verified = 0;
  for (let i = 0; i < N; i++) {
    if (validate(validSrc, validTrans)) verified++;
  }
  const t1 = process.hrtime.bigint();
  const totalMs = Number(t1 - t0) / 1e6;
  return { totalMs, checksPerSec: N / (totalMs / 1000), verified, n: N };
}

const b2 = bench2();
const b3 = bench3();
console.log(JSON.stringify({
  methodology: "real Node.js object-property-assignment merge (b2) and real regex-based slot/tag validator (b3), same operation counts as glossa's own spbench",
  node_version: process.version,
  benchmark2_json_merge: b2,
  benchmark3_regex_validator: b3,
}, null, 2));
