use documented::Documented;
use jsonrpsee::core::RpcResult;
use schemars::JsonSchema;
use serde::Serialize;
use transparent::address::TransparentAddress;
use zcash_address::{ToAddress, ZcashAddress};
use zcash_keys::encoding::AddressCodec;
use zcash_protocol::consensus::Parameters;

use crate::{components::json_rpc::server::LegacyCode, network::Network};

pub(crate) type Response = RpcResult<ResultType>;

/// The TEX address encoding of the input transparent P2PKH address.
#[derive(Clone, Debug, Serialize, Documented, JsonSchema)]
#[serde(transparent)]
pub(crate) struct ResultType(String);

pub(super) const PARAM_TRANSPARENT_ADDRESS_DESC: &str = "The transparent P2PKH address to convert.";

/// Converts a transparent P2PKH Zcash address to a TEX address.
///
/// # Arguments
/// - `params`: Network parameters for address encoding/decoding.
/// - `transparent_address`: The transparent P2PKH address to convert.
pub(crate) fn call(params: &Network, transparent_address: &str) -> Response {
    let decoded = TransparentAddress::decode(params, transparent_address)
        .map_err(|_| LegacyCode::InvalidAddressOrKey.with_static("Invalid address"))?;

    let pubkey_hash = match decoded {
        TransparentAddress::PublicKeyHash(hash) => hash,
        TransparentAddress::ScriptHash(_) => {
            return Err(LegacyCode::InvalidParameter
                .with_static("Address is not a transparent p2pkh address"));
        }
    };

    let tex_address = ZcashAddress::from_tex(params.network_type(), pubkey_hash);

    Ok(ResultType(tex_address.encode()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use zcash_protocol::{consensus, local_consensus::LocalNetwork};
    use zcash_protocol::consensus::{self, NetworkType};

    fn regtest() -> Network {
        Network::from_type(NetworkType::Regtest, &[])
    }

    // From zcash-test-vectors (transparent/zip_0320.py)
    const TEST_VECTORS: &[(&str, &str)] = &[
        (
            "tmQqjg2hqn5XMK9v1wtueg1CpzGbgTNGZQu",
            "texregtest15hhh9uprfp6krumprglg6qpx3928ulrnlxt9na",
        ),
        (
            "tmGiqpWKPJdraF2PqBzPojzkRbDE4fPTyAF",
            "texregtest1fns2jk8xpjr7rqtaggn2zpmcdtfyj2jer8arm0",
        ),
        (
            "tmEkTF6UovNsEQM9h1ehnA3byw6yhFCJWor",
            "texregtest1xulx2a0pgc84phkdtue67zwe26axtcvvyaf6yu",
        ),
        (
            "tmGoyC4XZ1GNCdJGk96K6mT8jxDQEhzVbfR",
            "texregtest1fhvw29vvg37mep5kkhyew47rrqadjtyk4xzx8n",
        ),
        (
            "tmG4gSmUZzCcyR6S5nBhEFrfmodmUjXXAZG",
            "texregtest1gk5swlnzf8m5hc9x82344aqv5s90k5x2dvqyvh",
        ),
        (
            "tmKgnRCv6SjwEFgXhqPoADKp3HLFF67Seww",
            "texregtest1d4j4uz8wnl5zmuzdl7y3fykrkk2zarnccfs578",
        ),
        (
            "tmTymg9bGECw8tR8WHepE45c4joNTUt1zth",
            "texregtest1epwdxsm94e9wh2zwad7j885gelxly2f8d4mqry",
        ),
        (
            "tmMGGBngBJwgTYWCx23zWDY7QvLateZNqCC",
            "texregtest106exvc6ufugdwppy7vnuf2vwztf5qh9r4tpsfa",
        ),
        (
            "tmUyukTGWjTM7Nw4j8zgbZZwPfM8enu9NvZ",
            "texregtest16ddmp690el6vrzajhftc3fqpmx4a3cgfqf97yn",
        ),
        (
            "tmDsJSojZxU3sb3LGMs6nMC1SVSQhKD99My",
            "texregtest19kgadp7hwu08xlufnvr2r0p5aygw3ss60px28u",
        ),
        (
            "tmWezycaJjPoXNJxK2zvzELjS65mzExnvVE",
            "texregtest1ukuxwrfrj20q65c25ssk3nkathsy8qneehv5vl",
        ),
        (
            "tmMUEbcXX7tVwtRjaSaMVfzXu6PCeyMnsCH",
            "texregtest1srmppx752ux07mntsjmthpddy2enunfuh6mlej",
        ),
        (
            "tmDjbRj8go7BrS8AxSjfjTBsCcHw7J45SCi",
            "texregtest19sw2lplpdswc4zvdtz3z8yt42wtz4recz5plym",
        ),
        (
            "tmSSJADGK9bgafaY9eih17WRazi3KzNL7RT",
            "texregtest1kacdt4amhphqx6tf4gla7a4qg8h9ey5tauz24v",
        ),
        (
            "tmSJ3JSRx2R71MfBksWzHNEfMxUgzMxDMxy",
            "texregtest1khs34j6m944un75ula8xxgvgdnjc4m2l0kfgpy",
        ),
    ];

    #[test]
    fn convert_test_vectors() {
        let params = regtest();
        for (input, expected) in TEST_VECTORS {
            let result = call(&params, input);
            assert!(result.is_ok(), "Failed to convert {}", input);
            let ResultType(tex) = result.unwrap();
            assert_eq!(&tex, *expected, "Mismatch for input {}", input);
        }
    }

    #[test]
    fn reject_invalid_address() {
        let params = regtest();
        let result = call(&params, "invalid_address");
        let err = result.unwrap_err();
        assert_eq!(err.code(), LegacyCode::InvalidAddressOrKey as i32);
        assert_eq!(err.message(), "Invalid address");
    }

    #[test]
    fn reject_p2sh_address() {
        let params = Network::Consensus(consensus::Network::MainNetwork);
        // Mainnet P2SH address (starts with t3)
        let result = call(&params, "t3Vz22vK5z2LcKEdg16Yv4FFneEL1zg9ojd");
        let err = result.unwrap_err();
        assert_eq!(err.code(), LegacyCode::InvalidParameter as i32);
        assert_eq!(err.message(), "Address is not a transparent p2pkh address");
    }
}
