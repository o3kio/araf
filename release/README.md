# Araf release contract

`manifest.schema.json` defines the machine-readable release tuple consumed by
downstream integrations. `scripts/publish-release.sh` emits one manifest for
each candidate and includes it in the deploy bundle and GitHub Release.

The manifest binds the release tag and source commit to the immutable OCI
indexes for the Tenant Console, Operator Console, Tenant BFF and Operator BFF.
The two BFF entries intentionally reference the same canonical `araf-bff`
multi-binary image and select their surface with the deployment command.

The compatibility section records supported backend profiles and the contract
versions required by a consumer. `fixture_mode_allowed` is always `false` for
release artifacts; fixture mode is available only through explicitly selected
development/test profiles.

GitHub Release publication remains owned by
`.github/workflows/release-publish.yml`. The image workflow only builds,
scans, attests and publishes OCI artifacts, with a preflight that rejects a
tag or image identity already present in the registry.
