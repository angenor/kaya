//! Écrit le contrat OpenAPI sur la sortie standard.
//!
//! # Pourquoi un binaire plutôt qu'un appel à `/api-docs/openapi.json`
//!
//! Le contrat servi par l'endpoint et celui imprimé ici sont **le même document** : les deux
//! viennent de `application::contrat_complet()`. Passer par le serveur imposerait à la
//! régénération du client — donc à la porte P-01 — de disposer d'une base de données, d'un port
//! libre et d'une attente de démarrage. Trois occasions d'échec intermittent sur une porte qui
//! doit être stable, faute de quoi elle finira désactivée.
//!
//! L'endpoint HTTP reste exposé : c'est lui que consomme Swagger UI et tout outil externe.

fn main() {
    let contrat = kaya_api::application::contrat_complet();

    // `to_json` sérialise dans l'ordre trié des chemins et des schémas — ni `preserve_order` ni
    // `preserve_path_order` n'étant activées sur utoipa (voir `backend/Cargo.toml`). C'est ce qui
    // donne les deux propriétés exigées par le gel §3.2 avant de clore US5 : même contrat, mêmes
    // octets ; et un endpoint ajouté en fin de fichier Rust produit un diff **local**, pas un
    // remaniement complet du fichier généré.
    match contrat.to_pretty_json() {
        Ok(json) => println!("{json}"),
        Err(erreur) => {
            eprintln!("sérialisation du contrat impossible : {erreur}");
            std::process::exit(1);
        }
    }
}
