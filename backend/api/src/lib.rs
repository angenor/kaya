//! `kaya-api` — binaire Actix du serveur, du nœud de site et du paquet auto-hébergé.
//!
//! **Un seul binaire, trois configurations** (constitution, § Pile technique imposée). Jamais
//! trois produits : ce qui diverge est un fichier de configuration, pas une base de code.

#![forbid(unsafe_code)]
