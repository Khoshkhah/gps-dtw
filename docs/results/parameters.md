# Effect of the parameters

Each table changes one parameter from the defaults. The values are recall / precision on a sample: every third route
(51 traces for each GPS interval). The defaults give these values on the sample:

| GPS interval | 1 s | 5 s | 15 s | 30 s |
|---|---|---|---|---|
| defaults | 0.963 / 0.982 | 0.959 / 0.958 | 0.937 / 0.941 | 0.908 / 0.916 |

## Mode

| Mode | 1 s | 5 s | 15 s | 30 s |
|---|---|---|---|---|
| **segment (default)** | 0.963 / 0.982 | 0.959 / 0.958 | 0.937 / 0.941 | 0.908 / 0.916 |
| node | 0.960 / 0.987 | 0.957 / 0.968 | 0.949 / 0.963 | 0.911 / 0.917 |

## Weight of the real GPS points (`gps_weight`)

| `gps_weight` | 1 s | 5 s | 15 s | 30 s |
|---|---|---|---|---|
| 1 | 0.959 / 0.980 | 0.952 / 0.967 | 0.931 / 0.945 | 0.897 / 0.919 |
| 2 | 0.958 / 0.979 | 0.953 / 0.966 | 0.931 / 0.945 | 0.900 / 0.918 |
| **5 (default)** | 0.963 / 0.982 | 0.959 / 0.958 | 0.937 / 0.941 | 0.908 / 0.916 |
| 10 | 0.964 / 0.983 | 0.963 / 0.948 | 0.940 / 0.923 | 0.912 / 0.909 |

A higher weight pulls the route through the real GPS points. Above 5, a noisy GPS point pulls the route onto a wrong
road near it.

## Time scale of the confidence weight (`tau`)

| `tau` | 1 s | 5 s | 15 s | 30 s |
|---|---|---|---|---|
| 1 s | 0.964 / 0.983 | 0.962 / 0.936 | 0.943 / 0.903 | 0.908 / 0.889 |
| **3 s (default)** | 0.963 / 0.982 | 0.959 / 0.958 | 0.937 / 0.941 | 0.908 / 0.916 |
| 6 s | 0.963 / 0.982 | 0.957 / 0.960 | 0.931 / 0.943 | 0.876 / 0.909 |
| 10 s | 0.963 / 0.982 | 0.957 / 0.961 | 0.927 / 0.939 | 0.835 / 0.892 |

## Bearing weight (`bearing_weight`, segment mode)

| `bearing_weight` | 1 s | 5 s | 15 s | 30 s |
|---|---|---|---|---|
| **0 (default)** | 0.963 / 0.982 | 0.959 / 0.958 | 0.937 / 0.941 | 0.908 / 0.916 |
| 0.05 | 0.959 / 0.979 | 0.960 / 0.964 | 0.937 / 0.943 | 0.906 / 0.920 |
| 0.1 | 0.959 / 0.980 | 0.960 / 0.966 | 0.939 / 0.947 | 0.906 / 0.921 |
| 0.2 | 0.958 / 0.976 | 0.961 / 0.967 | 0.937 / 0.946 | 0.898 / 0.918 |
| 0.5 | 0.953 / 0.953 | 0.962 / 0.969 | 0.932 / 0.945 | 0.884 / 0.913 |

The bearing helps at 5 s and 15 s. At 1 s, the bearing of a one-second step is noisy. At 30 s, the bearing of the
interpolated curve is often wrong.

## Search radius (`corridor_gap`)

| `corridor_gap` | 1 s | 5 s | 15 s | 30 s | Time |
|---|---|---|---|---|---|
| 0.5 | 0.963 / 0.982 | 0.883 / 0.959 | 0.842 / 0.934 | 0.668 / 0.826 | 4 ms |
| **1.0 (default)** | 0.963 / 0.982 | 0.959 / 0.958 | 0.937 / 0.941 | 0.908 / 0.916 | 6 ms |
| 1.5 | 0.974 / 0.981 | 0.959 / 0.958 | 0.944 / 0.950 | 0.917 / 0.921 | 7 ms |

With a smaller radius, the true road of a winding street can go out of the radius between two GPS points.

## Road point spacing (`speed`)

| `speed` | 1 s | 5 s | 15 s | 30 s |
|---|---|---|---|---|
| 8 m/s | 0.960 / 0.987 | 0.954 / 0.967 | 0.931 / 0.948 | 0.909 / 0.924 |
| **12 m/s (default)** | 0.963 / 0.982 | 0.959 / 0.958 | 0.937 / 0.941 | 0.908 / 0.916 |
| 16 m/s | 0.967 / 0.971 | 0.965 / 0.950 | 0.939 / 0.927 | 0.912 / 0.903 |

## Interpolation interval (`step`)

| `step` | 1 s | 5 s | 15 s | 30 s | Time |
|---|---|---|---|---|---|
| **1 s (default)** | 0.963 / 0.982 | 0.959 / 0.958 | 0.937 / 0.941 | 0.908 / 0.916 | 6 ms |
| 2 s | 0.939 / 0.937 | 0.953 / 0.963 | 0.933 / 0.944 | 0.906 / 0.917 | 2 ms |
| 3 s | 0.938 / 0.925 | 0.895 / 0.947 | 0.930 / 0.954 | 0.913 / 0.925 | 2 ms |
| 5 s | 0.941 / 0.892 | 0.560 / 0.894 | 0.890 / 0.935 | 0.899 / 0.923 | 1 ms |
| 5 s, node mode | 0.910 / 0.942 | 0.517 / 0.879 | 0.919 / 0.949 | 0.904 / 0.931 | 2 ms |

A larger step is faster, but the road points are further apart and the result is worse at 1 s to 15 s.

## Windows (`window`, `top_k`)

| Setting | 1 s | 5 s | 15 s | 30 s | Time |
|---|---|---|---|---|---|
| full DTW (`window` 0) | 0.962 / 0.982 | 0.959 / 0.958 | 0.944 / 0.951 | 0.871 / 0.854 | 15 ms |
| window 30, k 10 | 0.937 / 0.969 | 0.956 / 0.956 | 0.940 / 0.947 | 0.906 / 0.915 | 4 ms |
| window 60, k 10 | 0.954 / 0.978 | 0.957 / 0.957 | 0.933 / 0.934 | 0.908 / 0.918 | 5 ms |
| **window 60, k 20 (default)** | 0.963 / 0.982 | 0.959 / 0.958 | 0.937 / 0.941 | 0.908 / 0.916 | 6 ms |

The defaults give the result of the full DTW at 1 s to 15 s in less than half the time. At 30 s the full DTW is
worse: over the whole trace, the road can be more than `detour` × the interpolated curve, which cuts the corners, and
then the true road is not a candidate. In a window of 60 s that happens less. A short window or a small `k` can lose
the true road at a window border.

## Roads of a window (`detour`)

| `detour` | 1 s | 5 s | 15 s | 30 s | Time |
|---|---|---|---|---|---|
| 0 (all edges in the corridor) | 0.963 / 0.982 | 0.961 / 0.959 | 0.944 / 0.951 | 0.918 / 0.922 | 10 ms |
| 1.1 | 0.963 / 0.982 | 0.959 / 0.958 | 0.937 / 0.941 | 0.914 / 0.921 | 5 ms |
| **1.2 (default)** | 0.963 / 0.982 | 0.959 / 0.958 | 0.937 / 0.941 | 0.908 / 0.916 | 6 ms |
| 1.5 | 0.963 / 0.982 | 0.959 / 0.958 | 0.937 / 0.941 | 0.909 / 0.915 | 6 ms |

With the roads of a window from the two Dijkstra searches, the time is about half. The result is about 1 point lower
at 15 s and 30 s on the sample. With all edges in the corridor (0), a road with no path to the rest of the window can
still take road points.

## Turn restrictions (`turns`)

| Connections | 1 s | 5 s | 15 s | 30 s |
|---|---|---|---|---|
| all turns at a node | 0.963 / 0.982 | 0.958 / 0.957 | 0.936 / 0.937 | 0.903 / 0.912 |
| **duckOSM `edge_graph`** | 0.963 / 0.982 | 0.959 / 0.958 | 0.937 / 0.941 | 0.908 / 0.916 |
