# GPS-DTW

**Match a GPS trace to the road network with dynamic time warping: one second on both sides, all in Rust.**

GPS-DTW finds the road edges that a vehicle drove, from its GPS points. It interpolates the trace to one point each second,
divides the roads near the trace into points one second apart, and aligns the two sequences with dynamic time warping
([What DTW is](concepts/dtw.md)). Each input point has a confidence weight: a real GPS point counts fully, an interpolated point counts less.

<div class="hero-actions" markdown>

[Get started](start/index.md){ .md-button .md-button--primary }
[What DTW is](concepts/dtw.md){ .md-button }
[How it works](concepts/index.md){ .md-button }
[Results](results/index.md){ .md-button }

</div>

## Highlights

<div class="grid cards" markdown>

-   :material-timer-outline:{ .lg .middle } **One second on both sides**

    ---

    The input has a point each second, and the road a point each `speed × 1 s`. One DTW step is one second on both sides.
    [One second on both sides](concepts/time-division.md)

-   :material-scale-balance:{ .lg .middle } **Confidence weights**

    ---

    `w = 1 / (1 + (Δt / τ)²)`: an interpolated point far from a GPS point pulls the match less.
    [The confidence weight](concepts/confidence.md)

-   :material-vector-line:{ .lg .middle } **Segment or node mode**

    ---

    Compare segment with segment (the default), or point with point, with an optional bearing term.
    [Segment mode and node mode](concepts/cost.md)

-   :material-language-rust:{ .lg .middle } **All in Rust**

    ---

    5 to 23 ms for each 3-minute trace on one CPU core, with windows of one minute. Python calls one function.
    [Python API](reference/python.md)

</div>

## Results

600 simulated traces in Monaco. The values are recall / precision of the route edges (GPS-DTW: a sample of every third
route; the duckOSM HMM: all traces).

| GPS interval | GPS-DTW (segment mode) | duckOSM HMM |
|---|---|---|
| 15 s | 0.94 / 0.95 | 0.93 / 0.96 |
| 30 s | 0.92 / 0.92 | 0.90 / 0.96 |

GPS-DTW has the same recall as the HMM. Its precision is a little lower at 30 s. See [Results](results/index.md).

!!! note "State of the project"
    GPS-DTW is a research project. The values on this site come from the Monaco test data in [Test data](results/data.md).
