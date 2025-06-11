Typescript tests for the ts sdk.

## Setup

`pnpm install`

## Run

Before running the tests, make sure the `ts/sdk` rust crate has been rebuilt:

```sh
pushd ../sdk
make
popd
pnpm install
```

Then, start the local test validator with:

```sh
pnpm start:infra
```

Then, run the test script with:

```sh
pnpm test
```

After tests complete, teardown the local test validator with:

```sh
pnpm stop:infra
```

We do not use package.json's `pretest` and `posttest` scripts for this because `posttest` does not run if tests fail and cause the `test` command to exit with a nonzero code.
