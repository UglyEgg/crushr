# Format evolution

<div class="section-note"><strong>This page explains selection, not chronology.</strong> The point is not that time passed; the point is that multiple plausible design branches were tested and only a subset survived repeated corruption experiments.</div>

The story of crushr is not a straight line. It is a branching selection process in which multiple architectural ideas were tested under corruption and only a subset survived.

<div class="figure">
  <img src="assets/diagrams/crushr_master_diagram_final.svg" alt="crushr selection and elimination diagram" />
  <div class="caption">A better representation than a linear timeline: the current architecture emerged by eliminating weaker branches.</div>
</div>

## Milestone summary

| Experiment | Main question | Outcome |
|---|---|---|
| FORMAT-05 | Can payload blocks become self-identifying enough to improve salvage? | Yes. This became the foundational architectural pivot. |
| FORMAT-07 | Can salvage reason over verified relationships rather than flat metadata? | Yes. Recovery classification became more explicit and defensible. |
| FORMAT-08 | Do metadata placement strategies materially change survivability? | No. Fixed, hash, and golden placement were effectively tied. |
| FORMAT-09 / 10 | Do heavier metadata surfaces survive enough to justify their cost? | Largely no. Manifest-heavy paths remained expensive and less useful than hoped. |
| FORMAT-11 | Is distributed extent identity sufficient on its own? | Structurally yes, but not enough for strong named recovery. |
| FORMAT-12 | Can inline naming recover names without manifest-level overhead? | Yes, but repeated strings created measurable cost. |
| FORMAT-13 | Can dictionary indirection retain recovery and reduce size? | Yes. Header+tail dictionary mirroring emerged as the best balance. |
| FORMAT-14A | Does direct dictionary corruption validate the mirrored strategy? | Yes. Single dictionary was too fragile; header+tail behaved correctly. |

## Older phase visuals

### FORMAT-05 — the first architectural shift

<div class="figure">
  <img src="assets/diagrams/format05.png" alt="FORMAT-05 architectural shift" />
  <div class="caption">The earliest pivot: truth moved toward the payload instead of remaining entirely metadata-centric.</div>
</div>

### FORMAT-11 — structural recovery without strong naming

<div class="figure">
  <img src="assets/diagrams/format11.png" alt="FORMAT-11 comparison" />
  <div class="caption">FORMAT-11 proved distributed structural identity was cheap and useful, but not sufficient for strong named recovery on its own.</div>
</div>

### FORMAT-12 — naming restored, overhead exposed

<div class="figure">
  <img src="assets/diagrams/format12_named.png" alt="FORMAT-12 named recovery" />
</div>
<div class="figure">
  <img src="assets/diagrams/format12_overhead.png" alt="FORMAT-12 overhead" />
  <div class="caption">Inline naming was the bridge architecture: it restored names cheaply relative to manifests, but its duplication cost justified further optimization.</div>
</div>

### FORMAT-13 — dictionary indirection under stress

<div class="figure">
  <img src="assets/diagrams/format13.png" alt="FORMAT-13 stress size" />
  <div class="caption">The stress runs made the size hierarchy clear: dictionary indirection preserved the recovery model while materially reducing overhead.</div>
</div>

### FORMAT-14A — resilience under direct dictionary-target corruption

<div class="figure">
  <img src="assets/diagrams/format14a_resilience_corrected.png" alt="FORMAT-14A resilience" />
  <div class="caption">The decisive result: header+tail dictionary mirroring preserved named recovery when one copy survived and fell back anonymously when both were unavailable.</div>
</div>


### FORMAT-15 — refinement without promotion

FORMAT-15 explored two refinements to the winning mirrored-dictionary design:

- generation-aware dictionary identity
- factored namespace dictionaries

The submitted results were clear enough to make a decision.

On the baseline corpus, the factored variant preserved the same recovery behavior as `extent_identity_path_dict_header_tail` but increased archive size by 1,512 bytes.

On the stress corpus, it again preserved the same recovery behavior but increased archive size by 50,988 bytes.

That makes FORMAT-15 a useful negative result. The optimization attempt did not beat the current winner, so the lead architecture remains the non-factored header+tail mirrored dictionary model.
