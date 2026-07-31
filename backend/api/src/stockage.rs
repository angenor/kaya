//! Stockage objet — **consommé VIA L'API S3 UNIQUEMENT** (principe II).
//!
//! Garage est le service qui l'implémente en développement comme en production, mais rien ici ne
//! le nomme ni ne dépend de lui : le jour où un tenant auto-hébergé voudra du MinIO ou un
//! stockage géré, seule la configuration change.
//!
//! # Le seul objet stocké par ce cycle est le logo d'identité visuelle
//!
//! Il ne vit **jamais en base** : la table `branding` porte une clé d'objet. Un logo en base
//! gonflerait chaque sauvegarde, chaque réplication et chaque restauration, pour un fichier qui ne
//! change jamais.

use aws_sdk_s3::Client;
use aws_sdk_s3::config::{BehaviorVersion, Credentials, Region};
use aws_sdk_s3::primitives::ByteStream;

/// Échec d'accès au stockage objet.
#[derive(Debug, thiserror::Error)]
pub enum ErreurStockage {
    #[error("configuration du stockage absente : {0}")]
    Configuration(String),

    #[error("stockage objet indisponible : {0}")]
    Indisponible(String),
}

/// Accès au stockage objet.
#[derive(Debug, Clone)]
pub struct Stockage {
    client: Client,
    compartiment: String,
}

impl Stockage {
    /// Construit l'accès depuis l'environnement.
    ///
    /// # `force_path_style` n'est pas optionnel
    ///
    /// Le SDK AWS s'adresse par défaut à `<compartiment>.<hôte>`, ce qui suppose un DNS générique
    /// que ni Garage ni MinIO ne fournissent. Sans ce réglage, chaque requête part vers un nom
    /// d'hôte inexistant et échoue sur une résolution DNS — un message qui ne dit rien de la
    /// cause.
    pub fn depuis_environnement() -> Result<Self, ErreurStockage> {
        let variable = |nom: &str| -> Result<String, ErreurStockage> {
            std::env::var(nom).map_err(|_| ErreurStockage::Configuration(nom.to_owned()))
        };

        let endpoint = variable("S3_ENDPOINT")?;
        let region = variable("S3_REGION")?;
        let compartiment = variable("S3_BUCKET")?;
        let cle = variable("S3_ACCESS_KEY")?;
        let secret = variable("S3_SECRET_KEY")?;

        let config = aws_sdk_s3::Config::builder()
            .behavior_version(BehaviorVersion::latest())
            .region(Region::new(region))
            .endpoint_url(endpoint)
            .credentials_provider(Credentials::new(cle, secret, None, None, "kaya-env"))
            .force_path_style(true)
            .build();

        Ok(Self {
            client: Client::from_conf(config),
            compartiment,
        })
    }

    /// Téléverse un objet.
    ///
    /// La clé est fournie par l'appelant et **porte le tenant** : il est impossible de lire
    /// l'objet d'un autre client même en devinant le reste du chemin.
    pub async fn televerser(&self, cle: &str, contenu: Vec<u8>) -> Result<(), ErreurStockage> {
        self.client
            .put_object()
            .bucket(&self.compartiment)
            .key(cle)
            .body(ByteStream::from(contenu))
            .send()
            .await
            .map_err(|e| ErreurStockage::Indisponible(e.to_string()))?;

        Ok(())
    }

    /// Lit un objet — exposé pour le diagnostic et les tests.
    pub async fn lire(&self, cle: &str) -> Result<Vec<u8>, ErreurStockage> {
        let objet = self
            .client
            .get_object()
            .bucket(&self.compartiment)
            .key(cle)
            .send()
            .await
            .map_err(|e| ErreurStockage::Indisponible(e.to_string()))?;

        let octets = objet
            .body
            .collect()
            .await
            .map_err(|e| ErreurStockage::Indisponible(e.to_string()))?;

        Ok(octets.into_bytes().to_vec())
    }
}
