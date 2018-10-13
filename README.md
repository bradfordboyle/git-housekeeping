# git-housekeeping

## Concourse CI

```sh
fly --target=<target> set-pipeline \
    --config=pipeline.yml \
    --pipeline=git-housekeeping
```
