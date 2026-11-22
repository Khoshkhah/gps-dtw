# Effect of the parameters

Each table changes one parameter from the defaults. The values are recall / precision on a sample: every third route
(51 traces for each GPS interval). The defaults give these values on the sample:

| GPS interval | 1 s | 5 s | 15 s | 30 s |
|---|---|---|---|---|
| defaults | 0.963 / 0.982 | 0.961 / 0.959 | 0.944 / 0.951 | 0.918 / 0.922 |

## Mode

| Mode | 1 s | 5 s | 15 s | 30 s |
|---|---|---|---|---|
| **segment (default)** | 0.963 / 0.982 | 0.961 / 0.959 | 0.944 / 0.951 | 0.918 / 0.922 |
| node | 0.960 / 0.987 | 0.959 / 0.969 | 0.949 / 0.963 | 0.922 / 0.929 |

## Weight of the real GPS points (`gps_weight`)

| `gps_weight` | 1 s | 5 s | 15 s | 30 s |
|---|---|---|---|---|
| 1 | 0.959 / 0.980 | 0.955 / 0.968 | 0.940 / 0.956 | 0.906 / 0.924 |
| 2 | 0.958 / 0.979 | 0.956 / 0.968 | 0.939 / 0.956 | 0.910 / 0.923 |
| **5 (default)** | 0.963 / 0.982 | 0.961 / 0.959 | 0.944 / 0.951 | 0.918 / 0.922 |
| 10 | 0.964 / 0.983 | 0.965 / 0.949 | 0.947 / 0.933 | 0.920 / 0.912 |

A higher weight pulls the route through the real GPS points. Above 5, a noisy GPS point pulls the route onto a wrong
road near it.

## Time scale of the confidence weight (`tau`)

| `tau` | 1 s | 5 s | 15 s | 30 s |
|---|---|---|---|---|
| 1 s | 0.964 / 0.983 | 0.965 / 0.937 | 0.949 / 0.902 | 0.908 / 0.854 |
| **3 s (default)** | 0.963 / 0.982 | 0.961 / 0.959 | 0.944 / 0.951 | 0.918 / 0.922 |
| 6 s | 0.963 / 0.982 | 0.960 / 0.961 | 0.939 / 0.954 | 0.875 / 0.909 |
| 10 s | 0.963 / 0.982 | 0.959 / 0.963 | 0.935 / 0.950 | 0.822 / 0.879 |

## Bearing weight (`bearing_weight`, segment mode)

| `bearing_weight` | 1 s | 5 s | 15 s | 30 s |
|---|---|---|---|---|
| **0 (default)** | 0.963 / 0.982 | 0.961 / 0.959 | 0.944 / 0.951 | 0.918 / 0.922 |
| 0.05 | 0.959 / 0.979 | 0.962 / 0.966 | 0.944 / 0.954 | 0.916 / 0.926 |
| 0.1 | 0.959 / 0.980 | 0.963 / 0.967 | 0.946 / 0.958 | 0.913 / 0.926 |
| 0.2 | 0.958 / 0.976 | 0.964 / 0.968 | 0.946 / 0.957 | 0.901 / 0.922 |
| 0.5 | 0.951 / 0.953 | 0.966 / 0.970 | 0.937 / 0.954 | 0.879 / 0.913 |

The bearing helps at 5 s and 15 s. At 1 s, the bearing of a one-second step is noisy. At 30 s, the bearing of the
interpolated curve is often wrong.

## Search radius (`corridor_gap`)

| `corridor_gap` | 1 s | 5 s | 15 s | 30 s | Time |
|---|---|---|---|---|---|
| 0.5 | 0.963 / 0.982 | 0.897 / 0.961 | 0.845 / 0.935 | 0.674 / 0.831 | 6 ms |
| **1.0 (default)** | 0.963 / 0.982 | 0.961 / 0.959 | 0.944 / 0.951 | 0.918 / 0.922 | 11 ms |
| 1.5 | 0.974 / 0.981 | 0.961 / 0.959 | 0.944 / 0.951 | 0.918 / 0.921 | 17 ms |

With a smaller radius, the true road of a winding street can go out of the radius between two GPS points.

## Road point spacing (`speed`)

| `speed` | 1 s | 5 s | 15 s | 30 s |
|---|---|---|---|---|
| 8 m/s | 0.960 / 0.987 | 0.957 / 0.969 | 0.939 / 0.959 | 0.919 / 0.931 |
| **12 m/s (default)** | 0.963 / 0.982 | 0.961 / 0.959 | 0.944 / 0.951 | 0.918 / 0.922 |
| 16 m/s | 0.967 / 0.971 | 0.968 / 0.951 | 0.946 / 0.938 | 0.921 / 0.907 |

## Interpolation interval (`step`)

| `step` | 1 s | 5 s | 15 s | 30 s | Time |
|---|---|---|---|---|---|
| **1 s (default)** | 0.963 / 0.982 | 0.961 / 0.959 | 0.944 / 0.951 | 0.918 / 0.922 | 11 ms |
| 2 s | 0.939 / 0.937 | 0.956 / 0.964 | 0.937 / 0.949 | 0.916 / 0.926 | 4 ms |
| 3 s | 0.938 / 0.925 | 0.898 / 0.948 | 0.930 / 0.954 | 0.906 / 0.916 | 2 ms |
| 5 s | 0.941 / 0.892 | 0.560 / 0.894 | 0.905 / 0.952 | 0.894 / 0.921 | 2 ms |
| 5 s, node mode | 0.927 / 0.953 | 0.509 / 0.878 | 0.919 / 0.949 | 0.915 / 0.941 | 2 ms |

A larger step is faster, but the road points are further apart and the result is worse at 1 s to 15 s.

## Windows (`window`, `top_k`)

| Setting | 1 s | 5 s | 15 s | 30 s | Time |
|---|---|---|---|---|---|
| full DTW (`window` 0) | 0.962 / 0.982 | 0.961 / 0.959 | 0.944 / 0.951 | 0.918 / 0.921 | 30 ms |
| window 30, k 10 | 0.937 / 0.969 | 0.959 / 0.957 | 0.937 / 0.943 | 0.908 / 0.910 | 6 ms |
| window 60, k 10 | 0.954 / 0.978 | 0.960 / 0.959 | 0.941 / 0.946 | 0.910 / 0.917 | 11 ms |
| **window 60, k 20 (default)** | 0.963 / 0.982 | 0.961 / 0.959 | 0.944 / 0.951 | 0.918 / 0.922 | 11 ms |

The defaults give the same result as the full DTW in less than half the time. A short window or a small `k` can lose
the true road at a window border.

## Turn restrictions (`turns`)

| Connections | 1 s | 5 s | 15 s | 30 s |
|---|---|---|---|---|
| all turns at a node | 0.963 / 0.982 | 0.961 / 0.958 | 0.943 / 0.949 | 0.914 / 0.915 |
| **duckOSM `edge_graph`** | 0.963 / 0.982 | 0.961 / 0.959 | 0.944 / 0.951 | 0.918 / 0.922 |
