<div align="center">

<h1>Fear IR</h1>

</div>

<h2>Overview</h2>

<p>
Fear is an experimental research compiler project.
</p>

<hr>

<h3>Design</h2>

<pre>
Frontend
  ↓
SSA
  ↓
Optimizations
  ↓
Umbrella
  ↓
ISel
  ↓
RegAlloc
  ↓
Assembly
</pre>

<hr>

<h2>Features</h2>

<h3>SSA Intermediate Representation</h3>

<ul>
    <li>Strongly typed values</li>
    <li>Explicit control-flow graph</li>
    <li>Block parameters instead of phi nodes</li>
    <li>Dominator tree construction</li>
    <li>Dominance frontier analysis</li>
    <li>Easy serialization and debugging</li>
</ul>

<h3>Backend Infrastructure</h3>

<ul>
  <li>RTL intermediate representation (planned)</li>
  <li>Instruction selection (planned)</li>
  <li>Register allocation (planned)</li>
  <li>x86-64 backend (planned)</li>
</ul>

<hr>

<h2>Project Status</h2>

<p>
⚠ Work in Progress
</p>

<p>
Fear is currently under active development. Internal representations,
optimization passes, and backend components may change frequently.
</p>

<hr>

<h2>Roadmap</h2>

<ul>
    <li>[x] SSA IR</li>
    <li>[x] Dominator Analysis</li>
    <li>[x] Dominance Frontier Analysis</li>
    <li>[x] Mem2Reg</li>
    <li>[ ] Umbrella RTL</li>
    <li>[ ] Instruction selection</li>
    <li>[ ] Register Allocation</li>
    <li>[ ] x86-64 Backend</li>
    <li>[ ] C99/C11 Frontend</li>
</ul>

<hr>

<h2>Why Fear?</h2>

<p>
Fear is not intended to compete with LLVM. The project exists as a platform
for learning, experimentation, and exploration of modern compiler techniques,
including SSA construction, optimization passes, instruction selection,
register allocation, and backend development.
</p>

<hr>

<h2>License</h2>

<p>
MIT License
</p>