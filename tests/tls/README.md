### Generate the EC private key in SEC1 format

```sh
openssl ecparam -name prime256v1 -genkey -noout -out ec_key.pem
```

### Create a certificate signing request (CSR)

```sh
openssl req -new -key ec_key.pem -out ec_csr.pem
```

### Generate the self-signed certificate

```sh
openssl x509 -req -in ec_csr.pem -signkey ec_key.pem -out ec_cert.pem -days 365
```
