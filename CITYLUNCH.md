# Sujet

En te basant sur la description du projet CityLunch, implémentez plusieurs features d'une API REST. Les réponses de l'API doivent être au format JSON et être accompagnées de codes HTTP cohérents. Versionnez votre travail avec Git. Déposez le code source sur un dépôt GitHub public. L'utilisation de Symfony ou de Node.js sont recommandé, mais vous utiliserez le langage de programmation de votre choix. Soyez attentif aux "contraintes métiers" implicites, car leur implémentation sera évaluée.

## Partie 1

Créer les routes permettant :
1. d'ajouter, de modifier, de consulter et de supprimer des Produits
2. d'ajouter, de modifier, de consulter et de supprimer des Livreurs

Précisions : À ce stade, les routes d'API sont accessibles publiquement, sans authentification. Il n'est pas demandé d'implémenter un contrôle d'accès sur ces routes

## Partie 2

À la création du Livreur, celui-ci doit être notifié et recevoir son mot de passe de connexion. Développer également une route lui permettant de se connecter et de recevoir un token JWT.

Créer les routes protégées par JWP permettant aux livreurs :
1. d'ajouter des produits dans leur sac, d'en retirer et de consulter le contenu

## Partie 3

Vous devez implémenter un test unitaire afin de valider 1 règle métier de votre application.

Livrables attendus :
- Le code source de l'application, test compris
- Un modèle conceptuel de données des entités Produits, Livreurs, Sacs.
- Un export de vos routes d'API (Postman ou Bruno ou une documentation OpenAPI (Swagger)
- Un fichier readme.md précisant : les composants nécessaires au fonctionnement du projet, les commandes nécessaires à l'initialisation du projet par un développeur qui reprendrait le projet, l'adresse du dépôt GitHub, la commande nécessaire à l'exécution de votre test.

# CityLunch : Présentation générale

## Partie 1 : Les commandes

Sur son site web CityLunch propose à ses clients de commander les plats et les desserts du jour. Il paie au moment de la commande. Il n'y a pas de frais de livraison. Une fois la commande passée, le client reçoit ensuite une notification lorsqu'un livreur a pris sa commande. Il ne peux pas annuler sa commande. Une page lui indique toujours le suivi de sa commande. Il est informé du temps estimé avant livraison. En cas de non-livraison ou de livraison trop tardive (30 minutes de retard), le client est remboursé. Le client peut indiquer une appréciation des plats commandés.

## Partie 2 : Les livreurs

Chaque jour, les cuisiniers expériments de CityLunch préparent trois plats et deux desserts. Les plats changent chaque jour. Ces plats sont conditionnés à froid.

Les livreurs chargent plusieurs plats et desserts dans leur sac au moment de leur prise de services. Ces quantitiés sont enregistrés dans l'application. Ils circulent ensuite à vélo dans les rues en attendant d'être missionnés pour une livraison. Les livreurs peuvent repasser remplir leur sac au cours de leur service. À la fin de leur service, les livreurs redonnent les plats non livrés. Tous les mouvements de stocks sont enregistrés.

Le gérant crée les comtes des livreurs dans l'application.

## Les livraisons

Les livreurs peuvent signaler à l'application leur indisponibilité.

La position des livreurs est transmise toutes les dix minutes à l'application. Le gérant peut connaitre la position des livreurs, leur disponibilité ainsi que le contenu de leur sac.

Dès qu'un client a commandé, le gérant missione l'un des livreurs disponibles pour effectuer la livraison. Il peut décliner la mission ou l'accepter. Dans ce cas, il indique en retour le temps estimé pour effectuer la livraison, s'il ne répond pas, le gérant peut missionner un autre livreur.

Une commande doit être livrée par un livreur dans son intégralité (tous les plats commandés) ou pas du tout. Lorsqu'il a effectué sa livraison (ou s'il ne peut pas l'assurer), il l'indique dans l'application.
