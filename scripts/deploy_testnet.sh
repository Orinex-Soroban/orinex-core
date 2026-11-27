#!/usr/bin/env bash
# Deploys the primary contract to Stellar testnet, initializes it, and records
# the deployment in deployments/testnet.json.
#
# Required environment variables:
#   ADMIN         admin address (must sign the initialize invocation via IDENTITY)
#   TOKEN         SEP-41 asset contract address escrowed by the contract
#   FEE_RECIPIENT address that receives protocol fees
#
# Optional environment variables:
#   FEE_BPS         protocol fee in basis points (default 250 = 2.5%)
#   NETWORK         stellar network name (default testnet)
#   IDENTITY        stellar CLI key name (default orinex-testnet)
#   WASM            wasm path (default artifacts/primary.wasm)
#   DEPLOYMENTS_FILE deployment manifest (default deployments/testnet.json)
#   RPC_URL         RPC endpoint recorded in the manifest
#   PASSPHRASE      network passphrase recorded in the manifest
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

NETWORK="${NETWORK:-testnet}"
IDENTITY="${IDENTITY:-orinex-testnet}"
WASM="${WASM:-artifacts/primary.wasm}"
DEPLOYMENTS_FILE="${DEPLOYMENTS_FILE:-deployments/testnet.json}"
FEE_BPS="${FEE_BPS:-250}"
RPC_URL="${RPC_URL:-https://soroban-testnet.stellar.org}"
PASSPHRASE="${PASSPHRASE:-Test SDF Network ; September 2015}"

: "${ADMIN:?set ADMIN to the contract admin account address}"
: "${TOKEN:?set TOKEN to the SEP-41 asset contract address}"
: "${FEE_RECIPIENT:?set FEE_RECIPIENT to the fee recipient account address}"

if ! [[ "$FEE_BPS" =~ ^[0-9]+$ ]]; then
  echo "error: FEE_BPS must be a non-negative integer (got '${FEE_BPS}')" >&2
  exit 1
fi

if ! command -v stellar >/dev/null 2>&1; then
  echo "error: stellar CLI not found on PATH" >&2
  exit 1
fi

if [ ! -f "$WASM" ]; then
  bash scripts/build.sh
fi

if ! stellar keys address "$IDENTITY" >/dev/null 2>&1; then
  stellar keys generate "$IDENTITY" --network "$NETWORK" --fund
fi

DEPLOYER="$(stellar keys address "$IDENTITY")"
echo "network:  ${NETWORK}"
echo "deployer: ${DEPLOYER}"
echo "wasm:     ${WASM}"

CONTRACT_ID="$(stellar contract deploy \
  --wasm "$WASM" \
  --source "$IDENTITY" \
  --network "$NETWORK")"
echo "contract: ${CONTRACT_ID}"

stellar contract invoke \
  --id "$CONTRACT_ID" \
  --source "$IDENTITY" \
  --network "$NETWORK" \
  -- initialize \
  --admin "$ADMIN" \
  --token "$TOKEN" \
  --fee_bps "$FEE_BPS" \
  --fee_recipient "$FEE_RECIPIENT"

WASM_SHA256="$(sha256sum "$WASM" | cut -d' ' -f1)"
DEPLOYED_AT="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

mkdir -p "$(dirname "$DEPLOYMENTS_FILE")"
cat > "$DEPLOYMENTS_FILE" <<JSON
{
  "network": "${NETWORK}",
  "network_passphrase": "${PASSPHRASE}",
  "rpc_url": "${RPC_URL}",
  "contract_id": "${CONTRACT_ID}",
  "wasm": "${WASM}",
  "wasm_sha256": "${WASM_SHA256}",
  "admin": "${ADMIN}",
  "token": "${TOKEN}",
  "fee_bps": ${FEE_BPS},
  "fee_recipient": "${FEE_RECIPIENT}",
  "deployed_at": "${DEPLOYED_AT}",
  "deployed_by": "${DEPLOYER}"
}
JSON

echo "recorded deployment in ${DEPLOYMENTS_FILE}"
