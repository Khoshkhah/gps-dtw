# Effect of the parameters

The sample of every third route. Segment mode unless stated.

## Search radius

The radius = `max(corridor_min, corridor_gap × median distance between two GPS points)`.

| `corridor_gap` | Radius at 30 s | 15 s | 30 s | Time for each trace |
|---|---|---|---|---|
| 0.5 | about 134 m | 0.928 / 0.949 | 0.832 / 0.870 | 0.04 s |
| **1.0 (default)** | about 270 m | 0.939 / 0.954 | 0.908 / 0.919 | 0.07 s |
| 1.5 | about 400 m | 0.939 / 0.954 | 0.908 / 0.919 | 0.09 s |
| 2.0 | about 540 m | 0.939 / 0.954 | 0.908 / 0.919 | 0.11 s |

With 0.5, the true road of some 30 s traces goes out of the radius on a winding road, and the match fails almost
completely (15 traces cause half of the recall loss). A radius larger than the GPS spacing gives no more gain.

## Bearing weight at each GPS interval

With the defaults (`gps_weight` 5, the turn restrictions).

| Setting | 1 s | 5 s | 15 s | 30 s |
|---|---|---|---|---|
| node mode | 0.959 / 0.987 | 0.959 / 0.969 | 0.949 / 0.963 | 0.922 / 0.929 |
| segment, bearing 0 | 0.962 / 0.982 | 0.961 / 0.959 | 0.944 / 0.951 | 0.918 / 0.921 |
| segment, bearing 0.05 | 0.958 / 0.978 | 0.962 / 0.966 | 0.944 / 0.954 | 0.916 / 0.925 |
| segment, bearing 0.1 | 0.958 / 0.978 | 0.963 / 0.967 | 0.946 / 0.958 | 0.913 / 0.925 |
| segment, bearing 0.2 | 0.957 / 0.976 | 0.964 / 0.968 | 0.946 / 0.957 | 0.901 / 0.921 |
| segment, bearing 0.5 | 0.950 / 0.952 | 0.966 / 0.970 | 0.937 / 0.954 | 0.879 / 0.913 |

The bearing helps a little at 5 s and 15 s. At 1 s, the bearing of a one-second step is noisy. At 30 s, the bearing of
the interpolated curve is often wrong.

## Weight of the real GPS points

With the turn restrictions (duckOSM `edge_graph`).

| `gps_weight` | Segment 15 s | Segment 30 s | Node 15 s | Node 30 s |
|---|---|---|---|---|
| 1 | 0.940 / 0.956 | 0.906 / 0.924 | 0.930 / 0.963 | 0.903 / 0.931 |
| 2 | 0.940 / 0.956 | 0.910 / 0.923 | 0.936 / 0.964 | 0.908 / 0.928 |
| **5 (default)** | 0.944 / 0.951 | 0.918 / 0.921 | **0.949 / 0.963** | **0.922 / 0.929** |
| 10 | 0.947 / 0.933 | 0.920 / 0.911 | 0.950 / 0.945 | 0.926 / 0.921 |
| 20 | 0.955 / 0.930 | 0.924 / 0.905 | 0.947 / 0.927 | 0.928 / 0.913 |
| 50 | 0.948 / 0.892 | 0.907 / 0.833 | 0.946 / 0.899 | 0.914 / 0.848 |

A higher weight pulls the route through the real GPS points: the recall goes up. Above 5, the precision goes down,
because a noisy GPS point then pulls the route onto a wrong road near it. Node mode with `gps_weight = 5` is the best
setting.

## Turn restrictions

| Connections | Mode | 15 s | 30 s |
|---|---|---|---|
| all turns at a node | segment | 0.939 / 0.954 | 0.908 / 0.919 |
| duckOSM `edge_graph` | segment | 0.940 / 0.956 | 0.906 / 0.924 |
| all turns at a node | node | 0.929 / 0.962 | 0.899 / 0.929 |
| duckOSM `edge_graph` | node | 0.930 / 0.963 | 0.903 / 0.931 |

Without the turns, 12 of the 150 routes at 30 s used a forbidden turn.

## Bearing weight with the default radius

| Weight | 15 s | 30 s |
|---|---|---|
| 0 | 0.939 / 0.954 | 0.908 / 0.919 |
| 0.05 | 0.940 / 0.954 | 0.895 / 0.922 |
| 0.1 | 0.938 / 0.955 | 0.892 / 0.923 |
| 0.3 | 0.936 / 0.962 | 0.848 / 0.910 |

The tables below were measured with the old radius (`corridor_gap` 0.5).

## Mode

| Mode | 15 s | 30 s |
|---|---|---|
| Segment | 0.928 / 0.949 | 0.832 / 0.870 |
| Node | 0.917 / 0.957 | 0.823 / 0.885 |

## Bearing weight

| Weight | 180° counts as | 15 s | 30 s |
|---|---|---|---|
| 0 | | 0.928 / 0.949 | 0.832 / 0.870 |
| 0.1 | 18 m | 0.926 / 0.949 | 0.824 / 0.875 |
| 0.2 | 36 m | 0.926 / 0.951 | 0.815 / 0.875 |
| 0.5 | 90 m | 0.919 / 0.960 | 0.764 / 0.857 |
| 1.0 | 180 m | 0.919 / 0.970 | 0.762 / 0.862 |
| 2.0 | 360 m | 0.901 / 0.960 | 0.743 / 0.852 |

A high weight helps precision at 15 s and decreases recall at 30 s. In the middle of a 30 s gap, the direction of the
interpolated curve is often wrong.

## Time scale τ (node mode)

| τ | 15 s | 30 s |
|---|---|---|
| 3 s | 0.917 / 0.957 | 0.823 / 0.885 |
| 10 s | 0.887 / 0.948 | 0.719 / 0.849 |

## Speed (node mode)

| Speed | 15 s | 30 s |
|---|---|---|
| 8 m/s | 0.909 / 0.957 | 0.803 / 0.884 |
| 12 m/s | 0.917 / 0.957 | 0.823 / 0.885 |
