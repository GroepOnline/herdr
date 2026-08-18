# Herdr domain vocabulary

## Runtime tick

A runtime tick is one ordered cycle in which Herdr observes queued runtime activity, applies the resulting state changes, handles due time-based work, and decides whether a new presentation is needed. A tick is independent of whether Herdr is attached to a local terminal or serving a client.

## Runtime event

A runtime event is an occurrence presented to the runtime tick for processing. It may come from the application, the control interface, a user-facing client, or another runtime participant. The event's source is less important than the state change and presentation impact it causes.

## Render impact

Render impact describes how much of the current presentation must be refreshed after a runtime event or scheduled task: no presentation work, an update that can reuse retained presentation state, or a full recomputation.

## Runtime adapter

A runtime adapter connects a concrete execution surface to the runtime tick. A local terminal and a headless client server are different execution surfaces, but both provide input, handle surface-specific runtime events, and present the resulting render impact.
