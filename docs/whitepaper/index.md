<div class="hero">

# Whitepaper

<p class="lead">This section presents crushr as a technical format rather than an experiment diary. It explains what the format is for, how the archive is structured, what recovery guarantees it does and does not make, and why the current architecture emerged from the evaluation program.</p>

</div>

<div class="paper-meta">
  <div><strong>Audience</strong>Storage engineers, systems programmers, archival practitioners, and technical reviewers.</div>
  <div><strong>Claim</strong>crushr is a compression-oriented archive with deterministic salvage behavior under corruption.</div>
  <div><strong>Core mechanism</strong>Extent identity near the payload plus mirrored naming dictionaries.</div>
  <div><strong>Evaluation basis</strong>Deterministic destructive testing across competing design branches.</div>
</div>

## Executive summary

crushr isolates the minimum architecture required to keep verified data useful after damage. It treats structural identity, naming, and later metadata policy as distinct concerns instead of collapsing them into one fragile metadata surface. This produces a format that can remain useful when partially damaged while preserving strict integrity boundaries.

<div class="kpi-grid">
  <div class="kpi"><strong>Payload truth</strong>Local extent identity remains the primary recovery anchor.</div>
  <div class="kpi"><strong>Naming truth</strong>Mirrored dictionaries preserve names when one trusted copy survives.</div>
  <div class="kpi"><strong>Trust boundary</strong>Naming fails closed and falls back anonymously when proof is unavailable.</div>
</div>

## Whitepaper contents

<div class="page-links">
  <a class="page-link" href="problem-statement.md"><strong>Problem statement</strong>Why conventional archive assumptions break down under damage and what crushr is trying to solve.</a>
  <a class="page-link" href="architecture.md"><strong>Format architecture</strong>The current archive layout, extent identity model, and mirrored dictionary subsystem.</a>
  <a class="page-link" href="recovery-model.md"><strong>Recovery and integrity model</strong>How crushr classifies outcomes and why anonymous fallback matters.</a>
  <a class="page-link" href="evaluation.md"><strong>Experimental evaluation</strong>The evidence that selected the present architecture and eliminated the weaker branches.</a>
  <a class="page-link" href="applicability.md"><strong>Applicability and roadmap</strong>Where the format fits, what remains out of scope today, and how the next phases should proceed.</a>
</div>
