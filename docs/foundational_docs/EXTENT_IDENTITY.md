# Extent identity

Extent identity is the primary resilience mechanism in crushr. It allows recovery to begin with surviving payload rather than with a surviving global name map.

## What extent identity must answer

Each surviving extent must provide enough verified local truth to answer three questions without depending on the naming subsystem.

| Question | Why it matters |
|---|---|
| Which file does this extent belong to? | Required for grouping |
| Where does it belong in that file? | Required for ordered reconstruction |
| Does the payload still verify? | Required for trustworthy recovery |

## Design consequence

Because this truth lives with the extents, payload can remain useful even when the dictionary subsystem is lost or rejected. That is why crushr can degrade to anonymous recovery without discarding structurally verified data.

<div class="section-note">
  <strong>Important distinction.</strong> Extent identity is not a naming layer. It is the structural identity layer that makes naming optional rather than mandatory for useful recovery.
</div>
