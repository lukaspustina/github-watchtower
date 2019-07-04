use clams::config::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Config, Debug, Serialize, Deserialize, PartialEq)]
pub struct GitHubWatchTowerConfig {
    #[serde(rename = "pub_key")]
    pub pub_keys: Vec<PubKey>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PubKey {
    pub name: String,
    pub armored_key: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    use spectral::prelude::*;
    use toml;

    #[test]
    fn load_from_string() {
        let toml = r#"
            [[pub_key]]
            name = "Lukas Pustina"
            armored_key = """
                -----BEGIN PGP PUBLIC KEY BLOCK-----
                Comment: GPGTools - http://gpgtools.org

                mQINBFrsSLoBEACrxeDh1Cz5mlHbJUF7xaOBmdopMfqsWwtKOWpLM0872IFDzmgi
                /TNY3wuyfYoVqsgMlwJuXdKAo4XJeoXPY0AzPwTf5a51bYyZGHM6X+rjANbX43h4
                6ETUxyoKLFYRJYAvuHUXaviDOTQKJLqW+jzSTIGzgzBT/J0F0FzVxv5PDBe2a7vK
                W9v5vsvFrHVg5xBserpLaRzfevuNRE4nYD2a3Xv0cAX2xZKpC3+kkZ/UFEQiHXK9
                4aY5p8aDRWISFjd74fHdT/Rz8CZwNIoga6yTJRuPUS3e67Z2p/caiRMsxnvlwUVW
                IEFTQvl9cmgdx5rW5OjhFyP4m/Qdnh2Cazn86fZid6/eWRumn58iGNrjOVuij2qU
                QfNgYbJWriSyljmOLZ9WAD9UaBXv4h61UUxoQavNNIFPkm0jtvmI17HPGzeEIWp2
                k6Ls5zPdF1nN6w8yA0MVB/pS+fpB2LhFV9BNi9qZqrzZ9Lxuw+F8u/xty2J5Elld
                Q82oY9EyDGJvjuJTXgUlEphR/i90nt8kgVrPRhQgTcARJ3vkvloC2SlIU3eVljmY
                wKF2wmNAzRMJTE+S/XjcA0XCjcTE3kmCO1tT4WBazNWQxbuPNEZEkLqqjvGwZzea
                ruO5nWICUV6tyKeIJH6xbkqMXMW2XiPkycrWX92itIKCL0W0g54aMfPrhwARAQAB
                tCBMdWthcyBQdXN0aW5hIDxsdWthc0BwdXN0aW5hLmRlPokCVAQTAQgAPhYhBEFh
                DCZoU8bVf+GXYOz7XQMtgpESBQJa7EjJAhsDBQkG4/cABQsJCAcCBhUICQoLAgQW
                AgMBAh4BAheAAAoJEOz7XQMtgpES4KoP/AuCOTwHpCmKozH4/OV3JBkH0jOO6+wc
                6iavPxfFZNyLYRt1PA8DytWUn/3SF1g0QXD2VNpVEknsnIKe7u0TUeP+Un6TuiIw
                gY8hDE0C+UuSXPnRmz9YwkarxrmU9ElDzKuBuU2GM5jeVCRMEHuKgYF5mHTExjf+
                A5Bwl36KiaVXvGxUmmyqj0HfJVty0kvp7fZdYd2sVGMVt9vX8tfsdZbITw5ygPT4
                ha7VCCt5NkDo8jRL9ii5JnxNMX2e3n2riFAeacOdibEuttgvkYGlrKSDflGe4Xfn
                GN3fxdpHjU1dt1AWhWadnHuusxbgNtufyru5Dgx+qpypRYMJnKnvY16Q831N7Dk0
                s0VdQgcKvIlT3QRWkXv8aEWpVZx/B7URqfTPeHqbDcaW0N2cHm/9BdYEKF1y8roY
                tCi7C0jAVP0ojPmrqsVW5kD3+CMzxgYiKPYdp/dN0eIYhtFfT8ybIIEN8ZT1v0u2
                JRzl7JD7J5ww+T4KIBzTa0zuV7XT5q2brwAbTwd+eKxmhlw0ONBQwh4/xvcKF32N
                s3DsWgX9R+3wY3+mklX52CRYWK3BHzMiN9dPTj1tUlWQ1+LG+i8nDnQFETgV/r05
                ikc3/lzck2PM8ZQVJ7nN57sbBk/8wUT+DD4sOfPKh2uE8lJodHOHKNWBZqj1Ebp7
                uKyg3vzZ7IIttC5MdWthcyBQdXN0aW5hIDxsdWthcy5wdXN0aW5hQGNlbnRlcmRl
                dmljZS5jb20+iQJUBBMBCAA+FiEEQWEMJmhTxtV/4Zdg7PtdAy2CkRIFAlrsSOUC
                GwMFCQbj9wAFCwkIBwIGFQgJCgsCBBYCAwECHgECF4AACgkQ7PtdAy2CkRISeg//
                W4iKIecNLyvGohBIeoWLx6dzXZDSyYt/O1TydC2RAsEc0zvd5ACjzNiuZQ4g1t2J
                RRwr/RnKZ2qKNMTXIWmVvOlbDP3ICzR3k7w0+IB59kgnk6LdK2iZAu+c0F0NqLyG
                o2cTy1dvkAtv1JPvXbAOO5DdF3++/wB2jsWXuK4DDuKEMBbTqP7DVJowQSmSb/U9
                oelGvWQWqD67iUfRJtMZKGEAf3stGDa4lW5pcQUmsMQJ5ME6SynehZ8IU9zSIUHB
                eIINS/xiwilY1yqVsqtKd4ZeNECzCAAuqr9fs4wSJMVTcLnUU6WxVNuQJca9Fkwe
                evmF5ppA7b+mi0akO+Hohc7di5pkSaHkZ24j0I1XQdC0IFF+uhbYMwtLK7j1rtQV
                IWx0EV6YWP6XYyEmqRs2VT3mqSeqYL3i0Zj8VLN2D7JnckT7qNFrq5R+jexNkukQ
                ZUk4mHXl4C530yXXhyEnswGN819c+uYNJKbExp4LvIeVsr+K7hWmnxzPKXuHtXU8
                ZA3hWW77BLJFfJv4ZdBi9AHamoGL9nKE3WPVc49oP+COLranUdM8ZXo96PA3YBBd
                buZiqXDliIWERrKLYWWz06wMN/YrMg0GNFWY0R6FW7oIbj2cZ1986kTu27+8KA9k
                xRFYiwvdPktL2w0FpsUN37iqCijZnboqYXdVfsf/uEy0LEx1a2FzIFB1c3RpbmEg
                PGx1a2FzLnB1c3RpbmFAY29kZWNlbnRyaWMuZGU+iQJUBBMBCAA+FiEEQWEMJmhT
                xtV/4Zdg7PtdAy2CkRIFAlrsSRQCGwMFCQbj9wAFCwkIBwIGFQgJCgsCBBYCAwEC
                HgECF4AACgkQ7PtdAy2CkRJZ9w/+ImBqbKuudfzYqr9MoSvOhjya/jz6Z9abj0nP
                93Z3SSur9uF4QciLast9buFUhXz1WwIjMR9FPSne5D9nqzSyo5BknIEF4O3mVHi2
                06NdVIUlp4YY105tI+2DkhkRwUmCgDz9C1fCHDDw3Hrj/C5NBjgEfekMcqoWsTy0
                EgrrnGXm3bGc79u+O0b6hE/tsFL5Eh7N2X99yb0pHdR8JcqO/tWxSeMhuE9JQXIZ
                xRGE6iYUe/eXu2BSksicfUPi0jX+uaQfrWDkXSXpe/VbW4yPDbaVrRwtCyFlVV6s
                M6zmkFwEjYn07XPwAtaZlftEEX/+dYgeBt4mhIvhdzCU++K8snlABfRgX4+WxRx4
                lxV/KsRzHo9CxVrZKt85D23OVjZczSVKVoZ8g+GKDNgQCDaTleWwwsXKiMWBE1uU
                F2dA+5z+PRAOWURvHNGSG97uo684jj/42mXuMFg9zzaCQi+o5hDE/a+IM0OEMDgm
                r1icop/WHa5YkvQBcBp0JItoawu7LXPBP1DLBZZ5YXqco9SKilic8rPU9pTwXyLU
                qNld6gDDrjzb3o9vDBMYNKAnIiY7AwbYZ1rhxS12eG0NQ3/5DOzzEop3bDgu7usZ
                G/D9Xifj6GYTpJg6hBOsPfd+ze5xyZ535PwG20xZ+p9JpxGPoqK1qyzpSNAYtmXr
                LuqCvWS5Ag0EWuxIugEQAOPIBGeIi2kxJcmBw5YOVu7jl1b5YLj0SoGYFuPBEf+e
                dipryF/6gT8m3oHOlfd76wdsqzywXOH0ZapHhwyq+lj00Yo+Jaw7UIuIGVJpDGLr
                l1kugusKe1SgLcLi74by+8tQMtNapz3DhSWsWku1zMqud2ojvdm/vve/yAGASBhd
                kTK7arhH2hxb1Yi5ckXmvSdxjtWCyJGlPnwm98GcZy15jNjloGshkgxhzbOOqJN4
                VHW+rJXuxp3Vny7ZhdyweSAEbwfC0eRNmvUPtxKLfRlTOIwS4tHKzxrQgIijhBOt
                rDDhaLER+XN10ZK4IgnOo2QxaVUctDiIPmc7lDp6ZlW49TyhygjyYGpVmifMgX4/
                aYrYzSYzWkqQxg7t3nbVLSB4bjkvBh0PXfTRkzg5McwzeHE4+A2gbAeCdoFW06wr
                D9yPbw7moWeGHdl/a1kAEYJmJPTATHR6lSWI2xTI27jp1INP2CHFlzxEVu5hgQ/F
                4yHTVsbdJGV0omVTarU4R2ZbfNiSAAn714J1Opw0lHS9WXyl62J9WKIZhtq02v4q
                O13KkuH5sLhtNt8ErsHBluChI1cT/Pk93XzqzPmwzN3DwVQZ+DKs5I3plq+/cMO+
                +PKJN0AwIbV+2hbbNXwaJDkTwQzKujKN9Ua3yGAJp8Nrs7y8OSGyKxgJwPd2AvcL
                ABEBAAGJAjwEGAEIACYWIQRBYQwmaFPG1X/hl2Ds+10DLYKREgUCWuxIugIbDAUJ
                BuP3AAAKCRDs+10DLYKREskdEACHHHrLXEcQRdg7E5XHsg5ehK9Eg5XYrwn93b8z
                8vgucyI++Gbk6jEsN+F+t8+cnT52fjhKcc6ddLrlNPWWukLo8O+ZoAPCHuZ+Je8W
                N1UumtGaD6dhOhMhSzV6GyY3ca4WsVLAGvxiwXL/CQRxh1j6fdKZ+XxPtN17hPLZ
                ALAep5W5oBze5WzjL6L1WCJ8galikOXRM10dpiEeRZWCA1j0vTtQAo+RcsCOdR/P
                xYGWjlKv7DpriAP7+OUvpfLrbQpZPMnVXkHExezuE8cXBvFtgOBzfFLLzKIoXxpF
                6u4/bmH/Fp8dd1DV/ir3iDK1AV7fjF+8PP/lgfZTTGeiKRdp20qBOYIXfI5rnXdl
                VAjb1IijaYuo3mQ7RhGzecAZdyce8U1bUKsV4VDKrJ3h3EOg5g8y5fpU+mv0W5vM
                tyWVuyAZ8OKv1fBZWc88DBmoBLy04gcNHShLrMRZnVgEgddUNDvUjAwsEknWtvJD
                1Ros+R/8qi2JY0d/mhxlYYks4SDNHxRoT0o+xqfCjjiQOEkR2qJMfk+jeFy1EnSX
                Wa/oCsAg1bm4mkXJQwXPQb+fv6tOY6GXDixC1IjkW5pG6ATVhZBUhrgdcgUNtkN3
                q6F1cxO0zmoV+6h5XMsel5FRkm7d/KTvmM5mqoMMtyTQx1Y+Wi5oGjDCXwOJBpT5
                cI/LnA==
                =465K
                -----END PGP PUBLIC KEY BLOCK-----"""
            "#;

        let config: Result<GitHubWatchTowerConfig, _> = toml::from_str(&toml);

        asserting("loading config from toml successfully")
            .that(&config)
            .is_ok();
    }
}
