# programming_in_rust

Motivation
Webserver dienen im World Wide Web der Auslieferung von Inhalten an Klienten, die häufig über Webbrowser verbunden sind. Eine Facette dieser Aufgabe betrifft die Validierung der Nutzerauthentizität sowie die Forcierung von Zugriffseinschränkungen basierend auf der Nutzergruppe.

Ihre Aufgabe besteht in der Untersuchung der Eignung von Rust zur Implementation von Webservern mit dem Fokus auf der sprachseitigen Unterstützung bei der Implementation von Authentifikation und Zugangssteuerung.

Problemstellung
Implementieren Sie einen Webserver in Rust, der von einem Kreis authentifizierter Nutzer den Upload von Dateien zulässt und diese allen Nutzern unter den Dateinamen entsprechenden Pfaden zur Verfügung stellt.

Hinweise
Sie dürfen für die Implementation auf crates.io zurückgreifen, insbesondere sollten Sie sich dort das Webframework Rocket anschauen. Die Authentifizierung kann rudimentär erfolgen; es geht uns nur darum, wie man einen solchen Mechanismus umsetzen kann, nicht welchen.

Damit Ihnen der Einstieg leichter fällt, haben wir für Sie einen minimalen Webserver mit Rocket implementiert. Um diesen zu übersetzen müssen Sie auf die Nightly-Variante des Compilers umschwenken. Diesen installieren Sie zunächst mit dem Befehl rustup install nightly. Anschließend wechseln Sie in das entpackte Verzeichnis mit der Vorlage und wählen den Nightly-Compiler mit dem Befehl rustup override set nightly für das Webserver-Projekt aus. Danach sollte sich das Beispiel übersetzen und ausführen lassen und der Webserver unter http://localhost:8000 erreichbar sein. Für Ihre anderen Projekte ändert sich dadurch nichts.

Modalitäten
Bitte lösen Sie die Aufgabe und stellen Ihre Lösung am Freitag gemeinsam mit Ihren Gruppenpartnern vor. Beachten Sie bezüglich der Präsentation auch die Hinweise zu den Projekten. Reichen Sie Ihr Cargo-Projekt zusammen mit den Präsentationsfolien im PDF-Format in Form eines Zip-Archives anschließend hier ein.

Viel Spaß und viel Erfolg!