# ConnectionStatus

Mostra se o app está falando com a Tank-G e por onde.

- Props: `transport` (`usb` · `ble`), `state` (`connected` · `connecting` · `disconnected` · `error`), `device`, `detail` (ex. versão de firmware), `onClick` (vira botão para reconectar).
- Cada estado tem palavra própria; o ponto brilha em `ok` quando conectado e pulsa em `warn` ao conectar.
- Fica no canto superior direito do editor, sempre visível.
