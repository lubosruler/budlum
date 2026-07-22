#!/usr/bin/env bash
set -euo pipefail

required_tests=(
  base_fee_increase_is_bounded
  base_fee_decrease_is_bounded
  max_fee_below_base_fee_rejected
  effective_tip_cannot_exceed_priority_or_cap
  legacy_fee_maps_to_zero_tip
  reward_pool_default_schedule_valid
  reward_pool_conserves_budget
  reward_pool_rounding_remainder_deterministic
  total_bud_committed_counts_stake_and_unbonding
  supply_capacity_remaining_uses_committed_denominator
  legacy_fee_validation_uses_fee_market_gate
  priority_fee_accepted_when_within_max_fee
  max_fee_must_match_legacy_fee_during_migration
  fee_field_tampering_invalidates_signature
  fee_distribution_burns_base_fee_and_pays_proposer
  fee_distribution_proposer_receives_tip_in_block_finalization
  fee_distribution_treasury_split_is_deterministic
  fee_distribution_rejects_underpriced
  fee_distribution_zero_treasury_rate
  fee_distribution_full_treasury_rate
  fee_distribution_large_fee_exercises_treasury
)

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

self_test() {
  local tmp
  tmp="$(mktemp)"
  trap "rm -f '$tmp'" EXIT
  for name in "${required_tests[@]}"; do
    printf 'test %s ... ok\n' "$name" >> "$tmp"
  done
  bash "$0" "$tmp" >/dev/null
  grep -v "${required_tests[0]}" "$tmp" > "$tmp.bad"
  if bash "$0" "$tmp.bad" >/dev/null 2>&1; then
    fail "self-test expected missing test to fail"
  fi
  echo "Economy invariant gate self-test OK"
}

check_log() {
  local log="$1"
  [[ -f "$log" ]] || fail "test log missing: $log"
  for name in "${required_tests[@]}"; do
    grep -Eq "test .*${name} .*ok|${name}.*ok" "$log" \
      || fail "required economy invariant test did not run/pass: $name"
  done
  echo "Economy invariant gate OK: ${#required_tests[@]} named tests observed."
}

if [[ "${1:-}" == "--self-test" ]]; then
  self_test
else
  [[ $# -eq 1 ]] || fail "usage: $0 <cargo-test-log> | --self-test"
  check_log "$1"
fi
