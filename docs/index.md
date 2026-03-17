<div class="hero">

# crushr

<p class="lead"><strong>crushr is a salvage-oriented archival format built for the failure case, not merely the happy path.</strong> It addresses a narrow but serious systems question: when an archive is damaged, what can still be proven and recovered without guessing?</p>

<div class="pill-row">
  <span class="pill">Distributed extent identity</span>
  <span class="pill">Mirrored naming dictionaries</span>
  <span class="pill">Fail-closed semantics</span>
  <span class="pill">Deterministic corruption evidence</span>
</div>

</div>

<div class="paper-meta">
  <div><strong>Format stance</strong>Integrity-first archive semantics with deterministic salvage classification.</div>
  <div><strong>Architectural core</strong>Distributed extent identity plus mirrored naming dictionaries.</div>
  <div><strong>Trust boundary</strong>Fail-closed naming with anonymous fallback when proof is unavailable.</div>
  <div><strong>Selection method</strong>Deterministic corruption experiments, not speculative design preference.</div>
</div>

<div class="figure">
  <img src="assets/diagrams/crushr_master_diagram_final.svg" alt="crushr format evolution master diagram" />
  <div class="caption">crushr’s architecture emerged through elimination. Several plausible branches were tested; only the branches that survived repeated corruption experiments remained in the mainline design.</div>
</div>

<div class="section-header">
  <h2>What makes crushr materially different</h2>
  <p>Most archive formats assume the container remains structurally intact. crushr assumes that real failure is uglier than that and treats post-damage reasoning as part of the design, not a separate apology after extraction breaks.</p>
</div>

<div class="card-grid">
  <div class="card">
    <h3>Distributed identity</h3>
    <p>The strongest durable truth travels with the extents themselves rather than living only in a single central authority.</p>
  </div>
  <div class="card">
    <h3>Mirrored naming dictionaries</h3>
    <p>Names are recoverable when one verified mirror survives and are refused when the naming subsystem cannot be trusted.</p>
  </div>
  <div class="card">
    <h3>Fail-closed semantics</h3>
    <p>When naming proof disappears, crushr does not improvise. It preserves verified payload and downgrades honestly to anonymous recovery.</p>
  </div>
  <div class="card">
    <h3>Evidence-backed design</h3>
    <p>The current architecture was selected through deterministic corruption experiments rather than style preference or metadata folklore.</p>
  </div>
</div>

<div class="thesis">
  <strong>The real claim.</strong> crushr is not trying to be a more decorative ZIP file. Its claim is that archive formats should preserve what can still be proven after damage and should clearly distinguish between structural truth, naming truth, and later metadata policy.
</div>

<div class="section-header">
  <h2>Read the site in this order</h2>
  <p>The front-door pages establish the purpose and the architectural claim. The whitepaper explains the argument. The foundational references supply the lower-level structures behind it.</p>
</div>

<div class="page-links">
  <a class="page-link" href="why-crushr.md"><strong>Why crushr</strong>Positioning, legitimacy, and where the format fits in the real archive landscape.</a>
  <a class="page-link" href="whitepaper/index.md"><strong>Whitepaper</strong>A coherent technical narrative covering the problem, architecture, recovery model, and evaluation story.</a>
  <a class="page-link" href="format-evolution.md"><strong>Format evolution</strong>The selection-and-elimination story that killed weaker branches and produced the current design.</a>
  <a class="page-link" href="foundational_docs/index.md"><strong>Foundational references</strong>Lower-level format and recovery documents for implementation-facing readers.</a>
</div>

<div class="section-note">
  <strong>Current architectural identity.</strong> At this stage crushr is best described as a compression-oriented archive format with distributed extent identity, mirrored naming dictionaries, deterministic recovery classes, and a strict refusal to invent names when naming proof is unavailable.
</div>
