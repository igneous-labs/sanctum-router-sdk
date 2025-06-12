# test-fixtures

## Notes

- All stake accounts are active with activationEpoch=0, deactivationEpoch=u64::MAX
- picosol vsa was reduced to 1k SOL stake so that we do not run into stake warmup limits (solana-test-validator starts out with 1M sol staked, so at most 90k SOL can be activated in epoch 0). This means vaidator stake is not consistent with pool/validator list state.
- lido's max stake validator at time of collection is of vote `8jxSHbS4qAnh5yueFp4D9ABXubKqMwXqF3HtdzQGuphp`
