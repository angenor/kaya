//! Consommateurs d'événements — **deux implémentations de démonstration**.
//!
//! Elles n'ont aucune fonction métier. Leur objet est de rendre l'idempotence **démontrable** :
//! chacune garde la trace de son dernier événement traité, et le test de redémarrage brutal
//! (`backend/tests/worker_redemarrage.rs`) constate qu'une republication produit l'effet d'une
//! seule présentation.
//!
//! **Pourquoi la trace est en mémoire ici.** Un consommateur de production la persisterait — sinon
//! un redémarrage du processus rejouerait tout l'historique. Ces deux-là sont des jouets de test :
//! leur mémoire disparaît avec le processus, et c'est justement ce qui permet au test de simuler
//! un redémarrage en reconstruisant l'objet. Un vrai consommateur qui copierait ce choix
//! réintroduirait le bug ; c'est écrit ici pour que personne ne le fasse.

use std::collections::HashSet;
use std::sync::Mutex;

use crate::{ErreurConsommation, EventConsumer, EvenementPublie};
use uuid::Uuid;

/// Compte les événements **distincts** qui lui sont présentés.
#[derive(Debug, Default)]
pub struct CompteurIdempotent {
    nom: &'static str,
    vus: Mutex<HashSet<Uuid>>,
}

impl CompteurIdempotent {
    pub fn nouveau(nom: &'static str) -> Self {
        Self {
            nom,
            vus: Mutex::new(HashSet::new()),
        }
    }

    /// Nombre d'événements distincts traités — **pas** le nombre de présentations.
    pub fn distincts(&self) -> usize {
        self.vus.lock().expect("verrou empoisonné").len()
    }

    pub fn a_vu(&self, id: Uuid) -> bool {
        self.vus.lock().expect("verrou empoisonné").contains(&id)
    }
}

#[async_trait::async_trait]
impl EventConsumer for CompteurIdempotent {
    fn nom(&self) -> &'static str {
        self.nom
    }

    async fn consommer(&self, evenement: &EvenementPublie) -> Result<(), ErreurConsommation> {
        // `HashSet::insert` renvoie `false` si l'élément était déjà présent : la deuxième
        // présentation ne produit donc aucun effet. C'est toute l'idempotence, et elle tient en
        // une ligne — à condition d'être écrite.
        self.vus
            .lock()
            .expect("verrou empoisonné")
            .insert(evenement.id);
        Ok(())
    }
}

/// Journalise chaque événement, sans état persistant.
///
/// Second consommateur pour vérifier que le worker les parcourt **tous** : un worker qui
/// s'arrêterait au premier passerait un test à consommateur unique.
#[derive(Debug, Default)]
pub struct JournaliseurEvenements {
    presentations: Mutex<usize>,
}

impl JournaliseurEvenements {
    pub fn nouveau() -> Self {
        Self::default()
    }

    /// Nombre de **présentations**, republications comprises. C'est l'écart avec
    /// [`CompteurIdempotent::distincts`] qui rend la republication visible dans le test.
    pub fn presentations(&self) -> usize {
        *self.presentations.lock().expect("verrou empoisonné")
    }
}

#[async_trait::async_trait]
impl EventConsumer for JournaliseurEvenements {
    fn nom(&self) -> &'static str {
        "journaliseur"
    }

    async fn consommer(&self, evenement: &EvenementPublie) -> Result<(), ErreurConsommation> {
        *self.presentations.lock().expect("verrou empoisonné") += 1;
        tracing::debug!(
            evenement.id = %evenement.id,
            evenement.type = %evenement.type_evenement,
            sequence = evenement.sequence_etablissement,
            "événement publié"
        );
        Ok(())
    }
}
