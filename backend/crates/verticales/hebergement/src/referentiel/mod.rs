//! Le référentiel de l'offre — types de chambre, chambres, formules, barèmes, plages.
//!
//! **Classe C** au registre §7.1 : rien ici n'est atteignable hors ligne. C'est ce que disent les
//! privilèges de la migration `0024`, et ce que la porte P-13 vérifie sur les neuf opérations.
//!
//! Trois couches, exactement la forme du module doré : [`modele`], [`repository`], [`service`].

pub mod modele;
pub mod repository;
pub mod service;

pub use modele::{
    CategorieVue, CreerCategorie, CreerFormule, CreerUnite, ErreurReferentiel, FamilleFormule,
    FormuleVue, ModifierCategorie, ModifierFormule, ModifierUnite, PalierVue, PlageDemandee,
    PlageVue, RegleConversionTaxe, StatutMenage, TempsRemiseEnEtat, UniteVue, heure_depuis_texte,
    heure_en_texte,
};
pub use service::{
    AGREGAT_CATEGORIE, AGREGAT_FORMULE, ServiceReferentiel, TYPE_CATEGORIE_TARIF_MODIFIE,
    TYPE_FORMULE_CREEE, TYPE_FORMULE_MODIFIEE, VERSION_SCHEMA_HEB,
};
