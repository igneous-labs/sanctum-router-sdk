Typescript tests for the ts sdk.

## Setup

`pnpm install`

## Run

Before running the tests, make sure the `ts/sdk` rust crate has been rebuilt:

```sh
cd ../sdk
make
```

Then, start the local test validator with:

```sh
docker compose -f ../../docker-compose-local-validator.yml up
```

Then, run the test script with:

```sh
pnpm test
```

After tests complete, teardown the local test validator with:

```sh
docker compose -f ../../docker-compose-local-validator.yml down
```
