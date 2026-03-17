<div class="hero">

# Why crushr

<p class="lead"><strong>crushr exists because conventional archive assumptions become brittle in the damage case.</strong> It is a serious attempt to make “what can still be proven” more useful than “the container is no longer coherent, good luck.”</p>

</div>

<div class="paper-meta">
  <div><strong>Primary audience</strong>Storage engineers, systems programmers, archival practitioners, and technical reviewers.</div>
  <div><strong>Problem class</strong>Damaged containers where meaningful payload survives more readily than the metadata that once explained it.</div>
  <div><strong>Position</strong>Not universal packaging. Legitimate archival utility for workflows where recovery honesty matters.</div>
</div>

Conventional archive formats generally optimize for intact containers. They are very good at packaging, transport, and clean extraction when metadata remains authoritative. They are much less good at the awkward middle state where metadata is truncated, overwritten, or internally inconsistent but a large portion of payload still survives.

<div class="thesis">
  <strong>crushr rejects both bad extremes.</strong> It does not promise miracle reconstruction, and it does not treat every damaged container as unrecoverable. Its promise is narrower and more credible: recover exactly what can still be proven and classify the result in an operationally honest way.
</div>

## Why the architecture is credible

<div class="two-up">
  <div class="mini-panel">
    <strong>What survived testing</strong>
    <p>Extent identity attached to payload, mirrored naming dictionaries, anonymous fallback, and fail-closed naming rules.</p>
  </div>
  <div class="mini-panel">
    <strong>What did not survive testing</strong>
    <p>Metadata-heavy branches, placement gimmicks, and single-dictionary designs as the lead architecture.</p>
  </div>
</div>

The strongest architectural choices in crushr were not selected because they sounded elegant. They survived because the deterministic corruption harness repeatedly showed that they remained useful while competing branches did not.

<div class="callout">
  <strong>The central result.</strong> Payload-adjacent structural truth matters much more than heavy centralized metadata duplication. Naming is still worth preserving, but it is best preserved through a mirrored dictionary subsystem rather than a bloated manifest-heavy control surface.
</div>

## Where crushr fits

crushr is a good fit wherever the cost of structural failure is higher than the cost of carrying a little more architectural discipline. That includes long-lived storage, archival research, digital preservation, corruption testing, and workflows that need deterministic post-damage reasoning rather than opaque extraction failure.

It is not intended to replace commodity archives for every routine workflow. The point is not ubiquity. The point is legitimacy: crushr addresses a real problem class with a format whose behavior is explicit, bounded, and evidence-backed.

## Why this deserves to be taken seriously

A format becomes legitimate when it can answer four questions cleanly:

| Question | crushr answer |
|---|---|
| What problem are you solving? | Verified recovery under damage without guessing. |
| What architecture solves it? | Distributed extent identity plus mirrored naming dictionaries. |
| Why should anyone trust that choice? | The architecture survived deterministic destructive testing. |
| What happens when proof disappears? | The format downgrades honestly to anonymous recovery. |

<div class="section-note">
  <strong>The important distinction.</strong> crushr is not asking users to trust a novel format because it sounds inventive. It is asking them to evaluate a bounded archive design with a clear thesis, an explicit trust boundary, and a documented selection process.
</div>
