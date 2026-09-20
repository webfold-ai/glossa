import { parse } from '@babel/parser';
import { parse as parseVueSfc } from '@vue/compiler-sfc';

// Verbatim from glossa's src/cli/mod.rs run_spbench_suite() sample content.
const sampleJsx = `
export default function Card({ title, count, user }) {
    return (
        <div className="card">
            <h1>{title}</h1>
            <p>Welcome back, {user.name}! You have {count} notifications.</p>
            <button placeholder="Filter notifications">Click <a href="/settings">here</a> to configure</button>
        </div>
    );
}
`;

const sampleVue = `
<template>
    <div class="user-panel">
        <h2>{{ $t('User Account') }}</h2>
        <p>Hello {{ user.name }}, status: {{ status }}</p>
        <button @click="refresh">Refresh Telemetry</button>
    </div>
</template>
`;

const COUNT = 5000; // matches glossa's default --count

function timeIt(fn, n) {
  const latenciesUs = [];
  const t0 = process.hrtime.bigint();
  for (let i = 0; i < n; i++) {
    const s = process.hrtime.bigint();
    fn(i);
    const e = process.hrtime.bigint();
    latenciesUs.push(Number(e - s) / 1000);
  }
  const t1 = process.hrtime.bigint();
  const totalMs = Number(t1 - t0) / 1e6;
  latenciesUs.sort((a, b) => a - b);
  const pct = (p) => latenciesUs[Math.min(latenciesUs.length - 1, Math.floor((p / 100) * latenciesUs.length))];
  return {
    totalMs,
    meanUs: totalMs * 1000 / n,
    p50Us: pct(50),
    p90Us: pct(90),
    p99Us: pct(99),
    throughputPerSec: n / (totalMs / 1000),
  };
}

// warm up JIT before measuring, same discipline glossa's own loop doesn't
// need (Rust has no JIT warmup phase) but Node.js does, so a fair baseline
// must warm up first or it unfairly penalizes the incumbent with cold-JIT cost.
for (let i = 0; i < 500; i++) {
  parse(sampleJsx, { sourceType: 'module', plugins: ['jsx'] });
  parseVueSfc(sampleVue, { filename: `warmup${i}.vue` });
}

const jsxResult = timeIt((i) => {
  parse(sampleJsx, { sourceType: 'module', plugins: ['jsx'] });
}, COUNT);

const vueResult = timeIt((i) => {
  parseVueSfc(sampleVue, { filename: `User${i}.vue` });
}, COUNT);

// glossa's own benchmark alternates i%2==0 ? jsx : vue across `count` total
// iterations (half jsx, half vue) -- combine the same way for a fair
// like-for-like comparison against glossa's single combined number.
const combinedMeanUs = (jsxResult.meanUs + vueResult.meanUs) / 2;
const combinedThroughput = 1_000_000 / combinedMeanUs;

console.log(JSON.stringify({
  methodology: "real @babel/parser + @vue/compiler-sfc timing, verbatim sample content from glossa's own spbench source, 500-iteration JIT warmup before measuring, 5000 measured iterations each",
  node_version: process.version,
  babel_parser: jsxResult,
  vue_compiler_sfc: vueResult,
  combined_mean_us_per_file: combinedMeanUs,
  combined_throughput_files_per_sec: combinedThroughput,
}, null, 2));
