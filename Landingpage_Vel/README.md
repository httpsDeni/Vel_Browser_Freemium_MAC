<div align="center">

# ⚡ Vel Browser — Freemium Landing Page

**Landing page moderna, responsiva e de alta performance desenvolvida para o Vel Browser (navegador minimalista para macOS escrito em Rust).**

[![React](https://img.shields.io/badge/React-18.2-61DAFB?style=flat-square&logo=react&logoColor=black)](https://react.dev/)
[![Vite](https://img.shields.io/badge/Vite-5.4-646CFF?style=flat-square&logo=vite&logoColor=white)](https://vitejs.dev/)
[![Tailwind CSS](https://img.shields.io/badge/Tailwind_CSS-3.4-38BDF8?style=flat-square&logo=tailwind-css&logoColor=white)](https://tailwindcss.com/)
[![Rust](https://img.shields.io/badge/Rust-Nativo-DEA584?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-emerald?style=flat-square)](LICENSE)

</div>

---

## 📌 Sobre o Projeto

Esta landing page foi construída especificamente para apresentar o **Vel Browser**, um navegador ultraleve para macOS com apenas **1.6 MB** de tamanho e consumo de CPU **3.8× menor que o Safari**.

A página inclui uma interface **Dark Glassmorphism**, comparativos de desempenho reais em tempo real, demonstração de capturas de tela do aplicativo e a estrutura do **Modelo Freemium** (Plano Grátis & Plano Pro por R$ 9,99/mês).

---

## 🚀 Tecnologias Utilizadas

- **[React 18](https://react.dev/)** — Biblioteca UI baseada em componentes reativos
- **[Vite 5](https://vitejs.dev/)** — Bundler e ambiente de desenvolvimento ultrarrápido
- **[Tailwind CSS v3](https://tailwindcss.com/)** — Framework utilitário de estilização responsiva
- **[Lucide React](https://lucide.dev/)** — Conjunto moderno de ícones vetoriais
- **Google Fonts** — Tipografia estilizada com *Plus Jakarta Sans*, *Inter* e *JetBrains Mono*

---

## ✨ Seções & Recursos da Landing Page

1. **Hero Section (Impacto Visual)**:
   - Destaque para as métricas reais: **1.6 MB** de app, **0.4% CPU** na interface, motor nativo de aceleração **Metal** e **VideoToolbox**.
   - Botões de CTA para download da versão gratuita (`.dmg`) e assinatura do Plano Pro.

2. **Demonstração com Capturas Reais**:
   - Menu interativo com screenshots reais extraídas da pasta `store/` do navegador:
     - 🖼️ Interface Chromeless sem distrações
     - 📑 Gerenciamento de abas nativo no AppKit
     - 🔍 Omnibox híbrido e seguro (filtra scripts `data:` e `javascript:`)
     - ⚡ Tabela de amostragem de CPU vs Safari
     - 🛡️ Bloqueador de anúncios nativo `adblock-rust`
     - 🧠 Economizador de memória de abas frias

3. **Galeria com Modal Lightbox**:
   - Visualização em alta resolução de capturas de tela com zoom expansível ao clicar.

4. **Comparativo de Desempenho (Side-by-Side)**:
   - Gráficos empíricos comparando uso de CPU e RAM durante a reprodução de vídeo 1080p (Safari 45.1% CPU vs Vel 11.9% CPU).

5. **Modelo Freemium & Tabela de Preços**:
   - **Plano Grátis (R$ 0/mês)**: Navegador completo, abas ilimitadas, sem expiração e sem anúncios forçados.
   - **Plano Pro (R$ 9,99/mês)**: Bloqueador de anúncios em Rust, economizador de memória, Picture-in-Picture (`Cmd+Shift+P`) e chaves offline.

6. **Validador de Chave de Apoiador (Modal)**:
   - Interface simulada para validação offline de chaves no formato `VEL-XXXXXXXX-CCCC` ou tokens do Lemon Squeezy.

7. **FAQ & Suporte**:
   - Perguntas frequentes interativas com suporte a atalhos e link direto para abertura de issues no GitHub.

---

## 🛠️ Como Executar Localmente

### Pré-requisitos
- **Node.js** v18 ou superior
- **npm** v10 ou superior

### Passos

1. Clone o repositório:
```bash
git clone https://github.com/httpsDeni/Landingpage_BROWSER_VEL.git
cd Landingpage_BROWSER_VEL
```

2. Instale as dependências:
```bash
npm install
```

3. Inicie o servidor de desenvolvimento:
```bash
npm run dev
```

4. Abra no navegador:
```text
http://localhost:3000
```

---

## 📦 Build para Produção

Para gerar o bundle otimizado de produção na pasta `dist/`:

```bash
npm run build
```

Para visualizar a versão de produção gerada:

```bash
npm run preview
```

---

## 📂 Estrutura de Pastas

```text
Landingpage_Vel/
├── public/
│   └── images/            # Capturas de tela reais do Vel Browser
├── src/
│   ├── App.jsx            # Componente principal da Landing Page
│   ├── index.css          # Estilos globais & diretivas Tailwind CSS
│   └── main.jsx           # Ponto de entrada do React
├── index.html             # HTML base com SEO meta tags
├── package.json           # Dependências e scripts npm
├── postcss.config.js      # Configuração do PostCSS
├── tailwind.config.js     # Configuração do Tailwind CSS
└── vite.config.js         # Configuração do bundler Vite
```

---

## 📄 Licença

Este projeto está licenciado sob a licença **MIT**. Veja o repositório principal para mais detalhes.
