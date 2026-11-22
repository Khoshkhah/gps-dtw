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

    4 to 31 ms for each 3-minute trace on one CPU core, with windows of one minute. Python calls one function.
    [Python API](reference/python.md)

</div>

## Results

600 simulated traces in Monaco, both methods with their defaults. Recall / precision of the route edges, and the median
time for each trace.

| GPS interval | GPS-DTW | duckOSM HMM | GPS-DTW time | HMM time |
|---|---|---|---|---|
| 1 s | 0.95 / 0.97 | 0.96 / 0.66 | 4 ms | 65 ms |
| 5 s | 0.96 / 0.96 | 0.96 / 0.90 | 6 ms | 58 ms |
| 15 s | 0.92 / 0.94 | 0.95 / 0.94 | 15 ms | 60 ms |
| 30 s | 0.90 / 0.91 | 0.94 / 0.95 | 31 ms | 65 ms |

GPS-DTW has a much higher precision at 1 s and 5 s, and is faster there (3.5 times at 1 s). The HMM has a little more recall, and is better and faster at 15 s and 30 s. See [Results](results/index.md).

!!! note "State of the project"
    GPS-DTW is a research project. The values on this site come from the Monaco test data in [Test data](results/data.md).
