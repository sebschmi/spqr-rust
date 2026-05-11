#ifndef SP_COMPRESS_H
#define SP_COMPRESS_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#include "spqr_tree.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct SpCompressHandle SpCompressHandle;

typedef struct {
    uint32_t u;
    uint32_t v;
    uint32_t original_edge_id;
} SpCompressInputEdge;

#define SP_COMPRESS_KIND_SERIES   1
#define SP_COMPRESS_KIND_PARALLEL 2

typedef struct {
    uint8_t  kind;
    uint8_t  _pad[3];
    uint32_t left;
    uint32_t right;
    uint32_t children_offset;
    uint32_t children_count;
} SpCompressNode;

typedef struct {
    uint32_t u;
    uint32_t v;
    uint32_t child;
} SpCompressCoreEdge;

#define SP_COMPRESS_TAG_BIT      0x80000000u
#define SP_COMPRESS_PAYLOAD_MASK 0x7FFFFFFFu

#define SP_COMPRESS_CHILD_IS_MACRO(c)  (((c) & SP_COMPRESS_TAG_BIT) != 0)
#define SP_COMPRESS_CHILD_IS_EDGE(c)   (((c) & SP_COMPRESS_TAG_BIT) == 0)
#define SP_COMPRESS_CHILD_AS_EDGE(c)   ((c))
#define SP_COMPRESS_CHILD_AS_MACRO(c)  ((c) & SP_COMPRESS_PAYLOAD_MASK)

typedef struct {
    uint32_t input_nodes;
    uint32_t input_edges;
    uint32_t core_nodes;
    uint32_t core_edges_count;
    uint32_t macro_count;
    uint32_t macro_series;
    uint32_t macro_parallel;
    uint32_t series_reductions;
    uint32_t parallel_reductions;
    uint32_t iterations;
    uint8_t  fully_sp_reducible;
} SpCompressStats;

typedef struct {

    const SpCompressNode* macros_ptr;
    uint32_t macros_len;

    const uint32_t* children_ptr;
    uint32_t children_len;

    const SpCompressCoreEdge* core_edges_ptr;
    uint32_t core_edges_len;

    const uint32_t* core_nodes_ptr;
    uint32_t core_nodes_len;

    const uint32_t* input_endpoints_ptr;
    uint32_t input_endpoints_len;

    SpCompressStats stats;
} SpCompressTreeView;

SpCompressHandle* sp_compress_ffi(
    uint32_t n_nodes,
    const SpCompressInputEdge* edges_ptr,
    uint32_t edges_len,
    const uint8_t* contractible_ptr,
    uint32_t contractible_len,
    uint8_t build_core_spqr
);

void sp_compress_free(SpCompressHandle* handle);

uint8_t sp_compress_success(const SpCompressHandle* handle);

SpCompressTreeView sp_compress_get_tree(const SpCompressHandle* handle);

const SpqrTree* sp_compress_get_core_spqr(const SpCompressHandle* handle);

const uint32_t* sp_compress_core_node_inv(const SpCompressHandle* handle, uint32_t* out_len);

struct SpqrResult* sp_compress_reconstruct_ffi(
    uint32_t n_nodes,
    const SpCompressInputEdge* edges_ptr,
    uint32_t edges_len,
    const uint8_t* contractible_ptr,
    uint32_t contractible_len);

typedef struct {
    uint64_t t_compress_us;

    uint64_t t_build_spqr_core_us;
    uint64_t t_reconstruct_us;
    uint64_t t_normalize_us;

    uint64_t t_canonicalize_us;
    uint64_t t_canon_root_us;
    uint64_t t_canon_node_order_us;
    uint64_t t_canon_edge_orient_us;
    uint64_t t_canon_move_root_us;

    uint64_t t_reconstruct_build_builder_us;
    uint64_t t_reconstruct_normalize_in_place_us;
    uint64_t t_reconstruct_finalize_us;
    uint64_t t_reconstruct_defensive_normalize_us;

    uint64_t t_core_remap_us;
    uint64_t t_core_graph_build_us;
    uint64_t t_core_spqr_raw_us;
    uint64_t t_handle_wrap_us;

    uint64_t t_total_us;

    uint64_t t_compress_input_edges_us;
    uint64_t t_compress_init_work_us;
    uint64_t t_compress_init_dirty_us;

    uint64_t t_compress_reduce_series_us;
    uint64_t t_compress_reduce_parallel_us;

    uint64_t t_compress_materialize_us;
    uint64_t t_compress_cleanup_us;
    uint64_t t_compress_canon_series_us;
    uint64_t t_compress_sort_core_edges_us;

    uint64_t t_compress_collect_core_nodes_us;
    uint64_t t_compress_stats_shrink_us;

    uint64_t t_spqr_self_loop_scan_us;
    uint64_t t_spqr_precheck_us;
    uint64_t t_spqr_split_multi_edges_us;
    uint64_t t_spqr_work_graph_us;

    uint64_t t_spqr_triconn_us;
    uint64_t t_spqr_relabel_us;
    uint64_t t_spqr_combine_us;
    uint64_t t_spqr_merge_us;

    uint64_t t_spqr_assemble_us;
    uint64_t t_spqr_tree_total_us;
} SpCompressTimings;

SpCompressHandle* sp_compress_timed_ffi(
    uint32_t n_nodes,
    const SpCompressInputEdge* edges_ptr,
    uint32_t edges_len,
    const uint8_t* contractible_ptr,
    uint32_t contractible_len,
    uint8_t build_core_spqr,
    SpCompressTimings* out_timings);

struct SpqrResult* sp_compress_reconstruct_with_timings_ffi(
    uint32_t n_nodes,
    const SpCompressInputEdge* edges_ptr,
    uint32_t edges_len,
    const uint8_t* contractible_ptr,
    uint32_t contractible_len,
    SpCompressStats* out_stats,
    SpCompressTimings* out_timings);

#ifdef __cplusplus
}
#endif

#endif
