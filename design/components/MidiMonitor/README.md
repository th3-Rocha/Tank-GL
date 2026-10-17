# MidiMonitor

Log das mensagens MIDI trocadas com o pedal, para mapear o protocolo e depurar.

- Props: `messages` (`{ time, dir: "in" | "out", bytes, text, highlight }`), `paused`, `onTogglePause`, `onClear`, `height`, `title`.
- OUT em âmbar, IN em ciano. Rola sozinho para o fim enquanto não está pausado.
- Marque com `highlight` a mensagem que corresponde ao parâmetro que o usuário está mexendo.
