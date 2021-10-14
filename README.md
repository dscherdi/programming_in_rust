# programming_in_rust

## Motivation
Webserver dienen im World Wide Web der Auslieferung von Inhalten an Klienten, die häufig über Webbrowser verbunden sind. Eine Facette dieser Aufgabe betrifft die Validierung der Nutzerauthentizität sowie die Forcierung von Zugriffseinschränkungen basierend auf der Nutzergruppe.

Ihre Aufgabe besteht in der Untersuchung der Eignung von Rust zur Implementation von Webservern mit dem Fokus auf der sprachseitigen Unterstützung bei der Implementation von Authentifikation und Zugangssteuerung.

## Problemstellung
Implementieren Sie einen Webserver in Rust, der von einem Kreis authentifizierter Nutzer den Upload von Dateien zulässt und diese allen Nutzern unter den Dateinamen entsprechenden Pfaden zur Verfügung stellt.

## Setup
1. `$ rustup override set nightly`
2. `$ cargo run`
3. Der Webserver ist nun unter https://127.0.0.1:8000 erreichbar