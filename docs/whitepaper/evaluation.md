# Experimental evaluation

crushr’s strongest claim is not that it has an unusual architecture. It is that the architecture was selected through deterministic destructive testing.

## Method

The project uses a bounded corruption harness that repeatedly creates archive variants, damages specific regions, and measures which recovery classes remain available. That allows competing design ideas to be compared on equal footing and makes it possible to reject plausible but weak branches.

<div class="callout">
  <strong>The important methodological point.</strong> crushr’s architecture was not chosen by intuition, nostalgia, or a generic preference for “more metadata.” It survived because repeated destructive tests showed that it remained useful while competing branches did not.
</div>

## Key evidence

### FORMAT-12 — naming restored without manifest-scale overhead

<div class="two-up">
  <div class="figure">
    <img src="../assets/diagrams/format12_named.png" alt="FORMAT-12 named recovery chart" />
  </div>
  <div class="figure">
    <img src="../assets/diagrams/format12_overhead.png" alt="FORMAT-12 overhead chart" />
    <div class="caption">Inline naming was the bridge architecture: it restored named recovery cheaply relative to manifests, but repeated strings still imposed measurable cost.</div>
  </div>
</div>

### FORMAT-13 — dictionary indirection reduced size materially

<div class="figure">
  <img src="../assets/diagrams/format13.png" alt="FORMAT-13 stress size chart" />
  <div class="caption">Dictionary indirection preserved the recovery model while reducing stress-case overhead materially relative to inline path identity and manifest-heavy designs.</div>
</div>

### FORMAT-14A — resilience validated the winning strategy

<div class="figure">
  <img src="../assets/diagrams/format14a_resilience_corrected.png" alt="FORMAT-14A resilience outcomes" />
</div>

| Variant | Result |
|---|---|
| Single dictionary | Too fragile under dictionary-target corruption |
| Header + tail mirrors | Preserved naming when one mirror survived |
| Both mirrors unavailable | Anonymous fallback behaved correctly |
| Inconsistent mirrors | Conflict detection and fail-closed naming behaved correctly |

<div class="section-note">
  <strong>What the evidence selected.</strong> The present design is coherent because the competing branches were forced to justify themselves under the same destructive conditions. The surviving architecture is the one that earned its keep.
</div>


### FORMAT-15 — refinement attempt did not beat the current winner

FORMAT-15 tested two refinements to the winning mirrored-dictionary architecture:

- generation-aware dictionary identity
- factored namespace dictionaries

The result was useful, but not promotional.

On the baseline corpus, `extent_identity_path_dict_factored_header_tail` preserved the same recovery behavior as `extent_identity_path_dict_header_tail` while increasing archive size from 32,104 bytes to 33,616 bytes, a delta of 1,512 bytes.

On the stress corpus, the same pattern held: the factored variant again preserved the same recovery behavior while increasing archive size from 9,189,560 bytes to 9,240,548 bytes, a delta of 50,988 bytes.

That means the attempted refinement did not justify replacing the current winner. FORMAT-15 should be read as a negative-but-useful result: the non-factored header+tail mirrored dictionary architecture remains the lead candidate.

<div class="section-note">
  <strong>Interpretation.</strong> FORMAT-15 did not weaken the current design. It validated that the current winner survived an optimization attempt without being displaced.
</div>
