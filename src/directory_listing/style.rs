// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

pub(crate) const STYLES: &str = r#"
:after, :before { box-sizing: border-box; }
html {
    color-scheme: light dark;
    -moz-osx-font-smoothing: grayscale;
    -webkit-font-smoothing: antialiased;
    min-width: 20rem;
    text-rendering: optimizeLegibility;
    -webkit-text-size-adjust: 100%;
    -moz-text-size-adjust: 100%;
    text-size-adjust: 100%
}
body {
    padding: 1rem;
    font-family: Consolas, 'Liberation Mono', Menlo, monospace;
    font-size: .75rem;
    max-width: 70rem;
    margin: 0 auto;
    font-weight: 400;
    line-height: 1.5
}
h1 {
    margin: 0;
    padding: 0;
    font-size: 1rem;
    line-height: 1.25;
    margin-bottom: 0.5rem;
}
table {
    width: 100%;
    table-layout: fixed;
    border-spacing: 0;
}
hr { border-style: none; border-bottom: solid 1px gray; }
table th, table td {
    padding: .15rem 0;
    white-space: nowrap;
    vertical-align: top
}
table th a, table td a {
    display: inline-block;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 95%;
    vertical-align: top;
}
table tr:hover td { background-color: rgba(200, 200, 200, 0.2); }
footer { padding-top: 0.5rem; }
table tr th { text-align: left; }

@media (max-width:30rem) {
    table th:first-child { width: 20rem; }
}
@media (prefers-color-scheme: dark) {
    table tr:hover td { background-color: rgba(0, 0, 0, 0.2); }
}
@media (prefers-color-scheme: light) {
    table tr:hover td { background-color: rgba(200, 200, 200, 0.2); }
}
"#;
