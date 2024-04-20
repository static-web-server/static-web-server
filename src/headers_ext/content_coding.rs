// Derives an enum to represent content codings and some helpful impls
macro_rules! define_content_coding {
    ($($coding:ident; $str:expr,)+) => {
        use hyper::header::HeaderValue;
        use std::str::FromStr;

        #[derive(Copy, Clone, Debug, Eq, PartialEq)]
        /// Values that are used with headers like [`Content-Encoding`](self::ContentEncoding) or
        /// [`Accept-Encoding`](self::AcceptEncoding)
        ///
        /// [RFC7231](https://www.iana.org/assignments/http-parameters/http-parameters.xhtml)
        pub enum ContentCoding {
            $(
                #[doc = $str]
                $coding,
            )+
        }

        impl ContentCoding {
            /// Returns a `&'static str` for a `ContentCoding`
            ///
            /// # Example
            ///
            /// ```
            /// use static_web_server::headers_ext::ContentCoding;
            ///
            /// let coding = ContentCoding::BROTLI;
            /// assert_eq!(coding.to_static(), "br");
            /// ```
            #[inline]
            pub fn to_static(&self) -> &'static str {
                match *self {
                    $(ContentCoding::$coding => $str,)+
                }
            }
        }

        impl From<&str> for ContentCoding {
            /// Given a `&str` returns a `ContentCoding`
            ///
            /// Note this will never fail, in the case of `&str` being an invalid content coding,
            /// will return `ContentCoding::IDENTITY` because `'identity'` is generally always an
            /// accepted coding.
            ///
            /// # Example
            ///
            /// ```
            /// use static_web_server::headers_ext::ContentCoding;
            ///
            /// let invalid = ContentCoding::from("not a valid coding");
            /// assert_eq!(invalid, ContentCoding::IDENTITY);
            ///
            /// let valid = ContentCoding::from("gzip");
            /// assert_eq!(valid, ContentCoding::GZIP);
            /// ```
            #[inline]
            fn from(s: &str) -> Self {
                ContentCoding::from_str(s).unwrap_or_else(|_| ContentCoding::IDENTITY)
            }
        }

        impl FromStr for ContentCoding {
            type Err = ();

            /// Given a `&str` will try to return a `ContentCoding`
            ///
            /// Different from `ContentCoding::from_str(&str)`, if `&str` is an invalid content
            /// coding, it will return `Err(())`
            ///
            /// # Example
            ///
            /// ```
            /// use static_web_server::headers_ext::ContentCoding;
            /// use std::str::FromStr;
            ///
            /// let invalid = ContentCoding::from_str("not a valid coding");
            /// assert!(invalid.is_err());
            ///
            /// let valid = ContentCoding::from_str("gzip");
            /// assert_eq!(valid.unwrap(), ContentCoding::GZIP);
            /// ```
            #[inline]
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $(
                        stringify!($coding)
                        | $str => Ok(Self::$coding),
                    )+
                    _ => Err(())
                }
            }
        }

        impl std::fmt::Display for ContentCoding {
            #[inline]
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                write!(f, "{}", match *self {
                    $(ContentCoding::$coding => $str.to_string(),)+
                })
            }
        }

        impl From<ContentCoding> for HeaderValue {
            fn from(coding: ContentCoding) -> HeaderValue {
                match coding {
                    $(ContentCoding::$coding => HeaderValue::from_static($str),)+
                }
            }
        }
    }
}

define_content_coding! {
    ANY; "*",
    BROTLI; "br",
    COMPRESS; "compress",
    DEFLATE; "deflate",
    GZIP; "gzip",
    IDENTITY; "identity",
    ZSTD; "zstd",
}

#[cfg(test)]
mod tests {
    use super::ContentCoding;
    use std::str::FromStr;

    #[test]
    fn to_static() {
        assert_eq!(ContentCoding::GZIP.to_static(), "gzip");
    }

    #[test]
    fn to_string() {
        assert_eq!(ContentCoding::DEFLATE.to_string(), "deflate".to_string());
    }

    #[test]
    fn from() {
        assert_eq!(ContentCoding::from("br"), ContentCoding::BROTLI);
        assert_eq!(ContentCoding::from("GZIP"), ContentCoding::GZIP);
        assert_eq!(ContentCoding::from("zstd"), ContentCoding::ZSTD);
        assert_eq!(ContentCoding::from("*"), ContentCoding::ANY);
        assert_eq!(ContentCoding::from("blah blah"), ContentCoding::IDENTITY);
    }

    #[test]
    fn from_str() {
        assert_eq!(ContentCoding::from_str("br"), Ok(ContentCoding::BROTLI));
        assert_eq!(ContentCoding::from_str("zstd"), Ok(ContentCoding::ZSTD));
        assert_eq!(ContentCoding::from_str("*"), Ok(ContentCoding::ANY));
        assert_eq!(ContentCoding::from_str("blah blah"), Err(()));
    }
}
