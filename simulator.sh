#!/bin/bash
# simulator.sh - The Raydium Pool Stress Tester

echo "Starting Raydium Devnet Simulator..."
echo "Press Ctrl+C to stop the spam!"

# Loop forever
while true; do
  echo "Launching fake pool..."
  solana transfer 7YttLkHDoNj9wyDur5pM1ejNaAvT9X4eqaYcHQqtj2G5 0.0001 --with-memo "initialize2" --url devnet --allow-unfunded-recipient
  
  echo "Waiting 3 seconds before next launch..."
  sleep 3
done
