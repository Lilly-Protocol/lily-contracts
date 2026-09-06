# Event compatibility policy

Lily Protocol events are a public interface consumed by indexers and other off-chain integrations. Changes to an event therefore require the same compatibility care as changes to a contract entry point or stored type.

## Compatibility rules

1. Existing event topic tuples are immutable. Do not rename, reorder, remove, or change the type of a published topic.
2. Existing payload fields are immutable. Do not rename, reorder, remove, or change the type or meaning of a published field.
3. New optional information must be additive. Prefer a new event with a new topic when older consumers cannot decode the expanded payload safely.
4. A breaking schema change requires a new versioned event. Include the version in the topic or payload and continue publishing the prior event during the documented migration window.
5. Never reuse an old topic for a payload with different semantics.

## Pull request requirements

Any pull request that changes an event must:

- list every affected topic and payload;
- state whether the change is additive or versioned;
- add or update assertions for the exact topic tuple and payload shape;
- explain how existing indexers continue to decode the event; and
- document any migration window for a versioned event.

Reviewers must reject event changes that alter an existing topic or payload without a versioned migration path.

## Current event families

The current contracts publish short-symbol topics for initialization and state transitions. These topics and their existing payload shapes are covered by this policy from the point this document is adopted. New contracts must follow the same rules.
