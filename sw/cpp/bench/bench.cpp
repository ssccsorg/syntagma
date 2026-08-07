// Tagma C++ benchmark suite.
//
// Mirrors the scenario families of the Rust benches
// (sw/rust/benches/bench.rs) with a hand-rolled steady_clock harness:
// each benchmark closure runs `iterations` times per round over
// `rounds` rounds, and the mean and standard deviation are reported in
// nanoseconds per call. A JSON summary is exported in the same shape as
// the Rust export_results.py.
//
// Uses CoordSpaceM (mmap), so this target is Unix only, mirroring the
// Rust mmap feature gate.

#include "tagma_core/coord.h"
#include "tagma_core/coord_cube.h"
#include "tagma_core/coord_path.h"
#include "tagma_core/coord_set_n.h"
#include "tagma_core/coord_space.h"
#include "tagma_core/coord_space_m.h"
#include "tagma_core/coord_space_n.h"
#include "tagma_geo/spatial.h"
#include "tagma_kv/coord_cube_kv.h"
#include "tagma_kv/coord_kv_n.h"
#include "tagma_kv/dyn_coord_kv.h"

#include <algorithm>
#include <array>
#include <chrono>
#include <cmath>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <fstream>
#include <string>
#include <utility>
#include <vector>

namespace {

// Accumulates benchmark results so the optimizer cannot elide the
// measured work.
std::size_t g_sink = 0;

struct Result {
  std::string name;
  double mean_ns;
  double stddev_ns;
};

std::vector<Result> g_results;

// Measures the mean ns per op() call. Each round times `iterations`
// calls; a single warmup call precedes each timed round.
template <typename F>
void bench(const std::string& name, int iterations, int rounds, F&& op) {
  std::vector<double> per_op_ns;
  per_op_ns.reserve(static_cast<std::size_t>(rounds));
  for (int r = 0; r < rounds; ++r) {
    op();  // warmup
    const auto start = std::chrono::steady_clock::now();
    for (int i = 0; i < iterations; ++i) op();
    const auto end = std::chrono::steady_clock::now();
    const double total_ns =
        std::chrono::duration<double, std::nano>(end - start).count();
    per_op_ns.push_back(total_ns / iterations);
  }
  double mean = 0.0;
  for (double value : per_op_ns) mean += value;
  mean /= static_cast<double>(rounds);
  double variance = 0.0;
  for (double value : per_op_ns) variance += (value - mean) * (value - mean);
  variance /= static_cast<double>(rounds);
  const double stddev = std::sqrt(variance);
  g_results.push_back({name, mean, stddev});
  std::printf("%-46s %12.1f ns/op  (stddev %10.1f)\n", name.c_str(), mean,
              stddev);
}

tagma::Coord coord(uint16_t index) {
  return tagma::Coord::from_index(index).value();
}

// count distinct 2D paths: (i % N_VALID, (i / N_VALID) % N_VALID).
std::vector<tagma::CoordPath<2>> paths_2d(int count) {
  const int kValid = tagma::Coord::kNValid;
  std::vector<tagma::CoordPath<2>> out;
  out.reserve(static_cast<std::size_t>(count));
  for (int i = 0; i < count; ++i) {
    const uint16_t a = static_cast<uint16_t>(i % kValid);
    const uint16_t b = static_cast<uint16_t>((i / kValid) % kValid);
    out.push_back(tagma::CoordPath<2>::from_array({coord(a), coord(b)}));
  }
  return out;
}

// count distinct 3D paths: (i % N_VALID, (i / N_VALID) % N_VALID, 0).
std::vector<tagma::CoordPath<3>> paths_3d(int count) {
  const int kValid = tagma::Coord::kNValid;
  std::vector<tagma::CoordPath<3>> out;
  out.reserve(static_cast<std::size_t>(count));
  for (int i = 0; i < count; ++i) {
    const uint16_t a = static_cast<uint16_t>(i % kValid);
    const uint16_t b = static_cast<uint16_t>((i / kValid) % kValid);
    out.push_back(
        tagma::CoordPath<3>::from_array({coord(a), coord(b), coord(0)}));
  }
  return out;
}

// ── core: CoordSpaceN ──────────────────────────────────────────────

void bench_csn2_insert_all() {
  const auto paths = paths_2d(10000);
  bench("csn2 insert all 10k", 1, 5, [&] {
    tagma::CoordSpaceN2<uint32_t> space;
    for (const auto& path : paths) space.place_path(path, 1);
    g_sink += space.len();
  });
}

void bench_csn2_get_all() {
  const auto paths = paths_2d(10000);
  tagma::CoordSpaceN2<uint32_t> space;
  for (const auto& path : paths) space.place_path(path, 1);
  bench("csn2 get all 10k", 3, 3, [&] {
    std::size_t sum = 0;
    for (const auto& path : paths) {
      const uint32_t* value = space.at_path(path);
      if (value) sum += *value;
    }
    g_sink += sum;
  });
}

void bench_csn2_overwrite_all() {
  const auto paths = paths_2d(10000);
  tagma::CoordSpaceN2<uint32_t> space;
  for (const auto& path : paths) space.place_path(path, 1);
  bench("csn2 overwrite all 10k", 3, 3, [&] {
    for (const auto& path : paths) space.place_path(path, 2);
    g_sink += space.len();
  });
}

void bench_csn2_fill_remove_all() {
  const auto paths = paths_2d(10000);
  bench("csn2 fill+remove all 10k", 1, 5, [&] {
    tagma::CoordSpaceN2<uint32_t> space;
    for (const auto& path : paths) space.place_path(path, 1);
    for (const auto& path : paths) space.vacate_path(path);
    g_sink += space.len();
  });
}

void bench_csn2_iter() {
  const auto paths = paths_2d(10000);
  tagma::CoordSpaceN2<uint32_t> space;
  for (const auto& path : paths) space.place_path(path, 1);
  bench("csn2 iter 10k entries", 3, 3, [&] {
    const auto entries = space.entries();
    g_sink += entries.size();
  });
}

void bench_csn2_mixed() {
  const auto paths = paths_2d(5000);
  bench("csn2 mixed 5k", 1, 5, [&] {
    tagma::CoordSpaceN2<uint32_t> space;
    for (int i = 0; i < 5000; ++i) space.place_path(paths[i], 1);
    std::size_t sum = 0;
    for (int i = 0; i < 5000; i += 2) {
      const uint32_t* value = space.at_path(paths[i]);
      if (value) sum += *value;
    }
    for (int i = 0; i < 5000; i += 2) space.vacate_path(paths[i]);
    g_sink += sum + space.len();
  });
}

// ── core: dense CoordSpace (N=1) ───────────────────────────────────

void bench_space_entry_or_insert() {
  const int kValid = tagma::Coord::kNValid;
  bench("space entry or_insert 10k", 1, 5, [&] {
    tagma::CoordSpace<uint32_t> space;
    for (int i = 0; i < 10000; ++i) {
      space.entry(coord(static_cast<uint16_t>(i % kValid))).or_insert(1);
    }
    g_sink += space.len();
  });
}

void bench_space_retain_half() {
  const int kValid = tagma::Coord::kNValid;
  bench("space fill+retain half 10k", 1, 5, [&] {
    tagma::CoordSpace<uint32_t> space;
    for (int i = 0; i < 10000; ++i) {
      space.place(coord(static_cast<uint16_t>(i % kValid)), 1);
    }
    space.retain(
        [](tagma::Coord c, uint32_t) { return c.index() % 2 == 0; });
    g_sink += space.len();
  });
}

// ── core: CoordSetN ────────────────────────────────────────────────

void bench_setn_insert_1000() {
  const auto paths = paths_2d(1000);
  bench("setn insert 1k n2", 1, 5, [&] {
    tagma::CoordSetN<2> set;
    for (const auto& path : paths) set.insert(path);
    g_sink += set.len();
  });
}

void bench_setn_union() {
  const auto a_paths = paths_2d(500);
  const auto b_paths = paths_2d(1000);
  tagma::CoordSetN<2> a;
  tagma::CoordSetN<2> b;
  for (const auto& path : a_paths) a.insert(path);
  for (const auto& path : b_paths) b.insert(path);
  bench("setn union 500+1k n2", 3, 3, [&] {
    const auto u = a.set_union(b);
    g_sink += u.len();
  });
}

void bench_setn_iter() {
  const auto paths = paths_2d(1000);
  tagma::CoordSetN<2> set;
  for (const auto& path : paths) set.insert(path);
  bench("setn iter 1k n2", 3, 3, [&] {
    const auto ps = set.paths();
    g_sink += ps.size();
  });
}

// ── core: CoordSpaceM (mmap) ───────────────────────────────────────

void bench_csm_insert_1000() {
  const auto paths = paths_3d(1000);
  bench("csm insert 1k n3", 1, 5, [&] {
    tagma::CoordSpaceM3<uint32_t> space;
    for (const auto& path : paths) space.place_path(path, 1);
    g_sink += space.len();
  });
}

void bench_csm_get_1000() {
  const auto paths = paths_3d(1000);
  tagma::CoordSpaceM3<uint32_t> space;
  for (const auto& path : paths) space.place_path(path, 1);
  bench("csm get 1k n3", 3, 3, [&] {
    std::size_t sum = 0;
    for (const auto& path : paths) {
      const uint32_t* value = space.at_path(path);
      if (value) sum += *value;
    }
    g_sink += sum;
  });
}

// ── tagma_geo ──────────────────────────────────────────────────────

void bench_cube_proximity() {
  // Vary the center per call so the query cannot be hoisted out of the
  // timed loop by the optimizer.
  const auto centers = paths_2d(50);
  std::size_t cursor = 0;
  bench("cube proximity r3 (49 paths)", 1000, 3, [&] {
    cursor = (cursor + 1) % centers.size();
    const tagma::CoordCube<2, 2, 1> cube =
        tagma::CoordCube<2, 2, 1>::from_path(centers[cursor]);
    std::size_t count = 0;
    for (const auto& path : tagma_geo::proximity(cube, 3)) {
      (void)path;
      count += 1;
    }
    g_sink += count;
  });
}

void bench_cube_bounding_box() {
  // Vary the ranges per call so the box iteration cannot be hoisted.
  const tagma::CoordCube<2, 2, 1> cube = tagma::CoordCube<2, 2, 1>::from_path(
      tagma::CoordPath<2>::from_array({coord(0), coord(0)}));
  std::vector<std::array<std::pair<uint16_t, uint16_t>, 2>> range_sets;
  for (int i = 0; i < 10; ++i) {
    const uint16_t lo = static_cast<uint16_t>(i * 10);
    range_sets.push_back(
        {{{lo, static_cast<uint16_t>(lo + 50)},
          {lo, static_cast<uint16_t>(lo + 50)}}});
  }
  std::size_t cursor = 0;
  bench("cube bounding box 51x51", 100, 3, [&] {
    cursor = (cursor + 1) % range_sets.size();
    std::size_t count = 0;
    for (const auto& path :
         tagma_geo::bounding_box(cube, range_sets[cursor])) {
      (void)path;
      count += 1;
    }
    g_sink += count;
  });
}

void bench_cube_proximity_hamming() {
  const auto centers = paths_2d(50);
  std::size_t cursor = 0;
  bench("cube proximity hamming r1", 1000, 3, [&] {
    cursor = (cursor + 1) % centers.size();
    const tagma::CoordCube<2, 2, 1> cube =
        tagma::CoordCube<2, 2, 1>::from_path(centers[cursor]);
    std::size_t count = 0;
    for (const auto& path : tagma_geo::proximity_hamming(cube, 1)) {
      (void)path;
      count += 1;
    }
    g_sink += count;
  });
}

void bench_cube_distances() {
  const auto a_paths = paths_2d(50);
  const auto b_paths = paths_2d(50);
  std::size_t cursor = 0;
  bench("cube distance metrics x100", 100, 3, [&] {
    cursor = (cursor + 1) % a_paths.size();
    std::uint64_t sum = 0;
    for (int i = 0; i < 100; ++i) {
      const tagma::CoordCube<2, 2, 1> a =
          tagma::CoordCube<2, 2, 1>::from_path(
              a_paths[(cursor + i) % a_paths.size()]);
      const tagma::CoordCube<2, 2, 1> b =
          tagma::CoordCube<2, 2, 1>::from_path(
              b_paths[(cursor + i + 7) % b_paths.size()]);
      sum += tagma_geo::hamming_distance(a, b);
      sum += static_cast<std::uint64_t>(
          tagma_geo::euclidean_distance_approx(a, b) * 1e9);
      sum += tagma_geo::manhattan_distance(a, b);
    }
    g_sink += static_cast<std::size_t>(sum);
  });
}

// ── tagma_kv ───────────────────────────────────────────────────────

void bench_kv_single_insert_static() {
  bench("kv static single insert", 100, 3, [&] {
    tagma_kv::CoordKVN<2> kv;
    kv.insert("hi", std::vector<uint8_t>(1, 1));
    g_sink += kv.len();
  });
}

void bench_kv_single_get_static() {
  // Vary the key per call so the lookup cannot be hoisted out of the
  // timed loop.
  tagma_kv::CoordKVN<2> kv;
  std::vector<std::string> keys;
  keys.reserve(100);
  for (int i = 0; i < 100; ++i) {
    const std::string key{static_cast<char>((i * 3) % 256),
                          static_cast<char>((i * 5) % 256)};
    keys.push_back(key);
    kv.insert(key, std::vector<uint8_t>(1, 1));
  }
  std::size_t cursor = 0;
  bench("kv static single get", 100000, 3, [&] {
    cursor = (cursor + 1) % keys.size();
    const auto value = kv.get(keys[cursor]);
    if (value) g_sink += (*value)[0];  // read the heap buffer
  });
}

void bench_kv_single_insert_dyn() {
  bench("kv dyn single insert", 100, 3, [&] {
    tagma_kv::DynCoordKV kv;
    kv.insert("hello", std::vector<uint8_t>(1, 1));
    g_sink += kv.len();
  });
}

void bench_kv_single_get_dyn() {
  tagma_kv::DynCoordKV kv;
  std::vector<std::string> keys;
  keys.reserve(100);
  for (int i = 0; i < 100; ++i) {
    const std::string key = "key" + std::to_string(i);
    keys.push_back(key);
    kv.insert(key, std::vector<uint8_t>(1, 1));
  }
  std::size_t cursor = 0;
  bench("kv dyn single get", 100000, 3, [&] {
    cursor = (cursor + 1) % keys.size();
    const auto value = kv.get(keys[cursor]);
    if (value) g_sink += (*value)[0];  // read the heap buffer
  });
}

void bench_kv_batch_insert_2k() {
  std::vector<std::string> keys;
  keys.reserve(2048);
  for (int i = 0; i < 2048; ++i) {
    keys.push_back(std::string{static_cast<char>(i % 256),
                               static_cast<char>((i / 256) % 256)});
  }
  bench("kv static batch insert 2k", 1, 5, [&] {
    tagma_kv::CoordKVN<2> kv;
    for (const auto& key : keys) kv.insert(key, std::vector<uint8_t>(1, 1));
    g_sink += kv.len();
  });
}

void bench_kv_spatial_proximity() {
  tagma_kv::CoordKVN<2> kv;
  for (int i = 0; i < 1000; ++i) {
    const std::string key{static_cast<char>((i * 7) % 256),
                          static_cast<char>((i * 13) % 256)};
    kv.insert(key, std::vector<uint8_t>(1, 1));
  }
  // Vary the query center per call so the query cannot be hoisted.
  const auto centers = paths_2d(50);
  std::size_t cursor = 0;
  bench("kv spatial proximity r2 (1k entries)", 100, 3, [&] {
    cursor = (cursor + 1) % centers.size();
    const auto results =
        tagma_kv::proximity<2, 2, 1>(kv, centers[cursor], 2);
    g_sink += results.size();
  });
}

void print_usage(const char* program) {
  std::printf(
      "Usage: %s [--json PATH] [--commit SHA] [--timestamp TS]\n"
      "  --json PATH    write a JSON summary of the results\n"
      "  --commit SHA   commit identifier recorded in the JSON\n"
      "  --timestamp TS timestamp recorded in the JSON\n",
      program);
}

}  // namespace

int main(int argc, char** argv) {
  std::string json_path;
  std::string commit = "unknown";
  std::string timestamp = "unknown";
  for (int i = 1; i < argc; ++i) {
    if (std::strcmp(argv[i], "--json") == 0 && i + 1 < argc) {
      json_path = argv[++i];
    } else if (std::strcmp(argv[i], "--commit") == 0 && i + 1 < argc) {
      commit = argv[++i];
    } else if (std::strcmp(argv[i], "--timestamp") == 0 && i + 1 < argc) {
      timestamp = argv[++i];
    } else if (std::strcmp(argv[i], "--help") == 0) {
      print_usage(argv[0]);
      return 0;
    }
  }

  std::printf("=== Tagma C++ Benchmark Suite ===\n");
  std::printf("commit: %s\n\n", commit.c_str());

  bench_csn2_insert_all();
  bench_csn2_get_all();
  bench_csn2_overwrite_all();
  bench_csn2_fill_remove_all();
  bench_csn2_iter();
  bench_csn2_mixed();
  bench_space_entry_or_insert();
  bench_space_retain_half();
  bench_setn_insert_1000();
  bench_setn_union();
  bench_setn_iter();
  bench_csm_insert_1000();
  bench_csm_get_1000();
  bench_cube_proximity();
  bench_cube_bounding_box();
  bench_cube_proximity_hamming();
  bench_cube_distances();
  bench_kv_single_insert_static();
  bench_kv_single_get_static();
  bench_kv_single_insert_dyn();
  bench_kv_single_get_dyn();
  bench_kv_batch_insert_2k();
  bench_kv_spatial_proximity();

  std::printf("\nsink: %zu\n", g_sink);

  if (!json_path.empty()) {
    std::ofstream out(json_path);
    out << "{\n";
    out << "  \"timestamp\": \"" << timestamp << "\",\n";
    out << "  \"commit\": \"" << commit << "\",\n";
    out << "  \"benchmarks\": {\n";
    for (std::size_t i = 0; i < g_results.size(); ++i) {
      const Result& result = g_results[i];
      out << "    \"" << result.name << "\": {\"mean_ns\": " << result.mean_ns
          << ", \"stddev_ns\": " << result.stddev_ns << "}";
      out << (i + 1 < g_results.size() ? ",\n" : "\n");
    }
    out << "  }\n}\n";
    std::printf("Exported %zu benchmarks to %s\n", g_results.size(),
                json_path.c_str());
  }
  return 0;
}
