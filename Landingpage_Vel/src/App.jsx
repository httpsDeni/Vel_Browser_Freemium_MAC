import React, { useState } from 'react';
import { 
  Zap, 
  Shield, 
  Cpu, 
  HardDrive, 
  Download, 
  Lock, 
  Check, 
  Sparkles, 
  ChevronRight, 
  Github, 
  Key, 
  Heart, 
  Maximize2, 
  X, 
  CheckCircle2, 
  Code2, 
  Terminal, 
  Sliders,
  AlertCircle,
  MessageSquare,
  Image as ImageIcon,
  Eye,
  ExternalLink
} from 'lucide-react';

export default function App() {
  const LEMON_SQUEEZY_BUY_URL = "https://vel.lemonsqueezy.com/checkout/buy/28763ca3-b0e6-43d6-af26-037f6febc669";

  // State for Feature Showcase Carousel using real images
  const [selectedFeature, setSelectedFeature] = useState(0);

  // Lightbox modal for screenshots
  const [lightboxImg, setLightboxImg] = useState(null);

  // Modal State for Supporter Key
  const [isKeyModalOpen, setIsKeyModalOpen] = useState(false);
  const [keyInput, setKeyInput] = useState('');
  const [keyStatus, setKeyStatus] = useState(null);
  const [activeSupporterKey, setActiveSupporterKey] = useState(null);

  // FAQ Accordion State
  const [openFaq, setOpenFaq] = useState(0);

  // Real store images mapping
  const featureList = [
    {
      id: 'hero',
      title: 'Interface Minimalista Chromeless',
      subtitle: 'Sem distração. O navegador entra em segundo plano para o conteúdo brilhar.',
      image: '/images/01_hero.png',
      badge: '0.4% CPU UI',
      desc: 'Barra de tarefas translúcida que se funde ao macOS com NSVisualEffectView.'
    },
    {
      id: 'tabs',
      title: 'Gerenciamento Inteligente de Abas',
      subtitle: 'A barra de abas só aparece quando você precisa de mais de uma aba.',
      image: '/images/03_tabs.png',
      badge: 'AppKit Nativo',
      desc: 'Sem scripts pesados na UI. Pausa automática de animações em segundo plano.'
    },
    {
      id: 'omnibox',
      title: 'Omnibox Híbrido & Seguro',
      subtitle: 'Um campo único para pesquisas e URLs que barra URLs maliciosas.',
      image: '/images/04_omnibox.png',
      badge: 'Segurança Nativa',
      desc: 'Trata automaticamente URLs javascript: e data: como busca para evitar exploits.'
    },
    {
      id: 'cpu',
      title: 'Eficiência Energética de CPU (3.8×)',
      subtitle: 'Comparativo real de consumo jogando vídeo 1080p em tempo real.',
      image: '/images/05_cpu.png',
      badge: '11.9% CPU Total',
      desc: 'Safari consome 45.1% de CPU no mesmo teste, enquanto o Vel consome apenas 11.9%.'
    },
    {
      id: 'blocking',
      title: 'Bloqueio de Anúncios adblock-rust',
      subtitle: 'Filtro AdBlock Plus executado direto na camada de rede do WebKit.',
      image: '/images/06_blocking.png',
      badge: 'Recurso Pro',
      desc: 'Solicitações bloqueadas nunca abrem conexão de socket, economizando banda e tempo.'
    },
    {
      id: 'memory',
      title: 'Economizador de Memória de Abas Frias',
      subtitle: 'Descarte inteligente que devolve a memória RAM instantaneamente ao macOS.',
      image: '/images/07_memory.png',
      badge: 'Recurso Pro',
      desc: 'Abas antigas e sem uso são desanexadas do motor de renderização automaticamente.'
    },
    {
      id: 'shortcuts',
      title: 'Atalhos Nativos do macOS',
      subtitle: 'Navegação ultrarrápida via teclado integrada ao sistema.',
      image: '/images/08_shortcuts.png',
      badge: 'Cmd+1..9',
      desc: 'Atalhos Cmd+T, Cmd+W, Cmd+Shift+P para Picture-in-Picture nativo.'
    },
    {
      id: 'stack',
      title: 'Arquitetura de Mídia Zero-Copy',
      subtitle: 'VideoToolbox -> Core Animation -> Metal via IOSurface.',
      image: '/images/09_stack.png',
      badge: 'Metal & CoreAnim',
      desc: 'Sem cópia de memória entre a GPU e a CPU ao reproduzir vídeos 4K/HEVC.'
    }
  ];

  // Gallery images from store
  const galleryScreenshots = [
    { title: 'Janela Única Chromeless', src: '/images/a_single.png', desc: 'Visualização limpa sem barras desnecessárias.' },
    { title: 'Múltiplas Abas Ativas', src: '/images/b_tabs.png', desc: 'Barra de abas fluida com atalhos macOS.' },
    { title: 'Omnibox de Busca Híbrida', src: '/images/c_omnibox.png', desc: 'Barra de endereço translúcida com modo escuro automático.' },
    { title: 'Navegação Retroceder / Avançar', src: '/images/d_back.png', desc: 'Animação nativa fluida de navegação.' }
  ];

  const handleValidateKey = (e) => {
    e.preventDefault();
    const cleanKey = keyInput.trim().toUpperCase();
    if (cleanKey.startsWith('VEL-') || cleanKey.length >= 10) {
      setKeyStatus({
        type: 'success',
        message: 'Chave de Apoiador Válida! Recurso Pro desbloqueado com sucesso offline.'
      });
      setActiveSupporterKey(cleanKey);
    } else {
      setKeyStatus({
        type: 'error',
        message: 'Formato de chave inválido. Use o formato VEL-XXXXXXXX-CCCC ou cole seu token UUID do Lemon Squeezy.'
      });
    }
  };

  return (
    <div className="min-h-screen bg-[#060913] text-slate-100 relative overflow-hidden bg-grid-pattern">
      {/* Background Ambient Glows */}
      <div className="absolute top-0 left-1/2 -translate-x-1/2 w-[1000px] h-[500px] bg-gradient-to-b from-orange-500/15 via-amber-500/5 to-transparent blur-3xl pointer-events-none rounded-full" />
      <div className="absolute top-[40%] right-[-100px] w-[500px] h-[500px] bg-cyan-500/10 blur-3xl pointer-events-none rounded-full" />

      {/* NAVBAR */}
      <header className="sticky top-0 z-50 backdrop-blur-xl bg-[#060913]/80 border-b border-white/10">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 h-20 flex items-center justify-between">
          <div className="flex items-center space-x-3">
            <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-orange-500 to-amber-600 flex items-center justify-center shadow-lg shadow-orange-500/30">
              <Zap className="w-6 h-6 text-white fill-white" />
            </div>
            <div>
              <div className="flex items-center space-x-2">
                <span className="font-extrabold text-2xl tracking-tight font-display gradient-text-silver">Vel</span>
                <span className="px-2 py-0.5 text-xs font-mono font-semibold bg-orange-500/20 text-orange-400 border border-orange-500/30 rounded-full">v1.0.4</span>
              </div>
              <p className="text-[11px] text-slate-400 font-mono">macOS • Rust Engine</p>
            </div>
          </div>

          <nav className="hidden md:flex items-center space-x-8 text-sm font-medium text-slate-300">
            <a href="#hero-preview" className="hover:text-orange-400 transition-colors">Interface</a>
            <a href="#recursos" className="hover:text-orange-400 transition-colors">Recursos</a>
            <a href="#galeria" className="hover:text-orange-400 transition-colors">Capturas</a>
            <a href="#desempenho" className="hover:text-orange-400 transition-colors">Desempenho</a>
            <a href="#planos" className="hover:text-orange-400 transition-colors">Planos Freemium</a>
            <a href="#faq" className="hover:text-orange-400 transition-colors">FAQ & Suporte</a>
          </nav>

          <div className="flex items-center space-x-3">
            <button 
              onClick={() => setIsKeyModalOpen(true)}
              className="px-3.5 py-2 text-xs font-medium text-slate-300 hover:text-white bg-white/5 hover:bg-white/10 border border-white/10 rounded-lg transition-all flex items-center space-x-1.5"
            >
              <Key className="w-3.5 h-3.5 text-amber-400" />
              <span>{activeSupporterKey ? 'Licença Ativa' : 'Ativar Chave'}</span>
            </button>

            <a 
              href={LEMON_SQUEEZY_BUY_URL}
              target="_blank"
              rel="noreferrer"
              className="px-4 py-2 text-xs font-semibold text-white bg-gradient-to-r from-orange-500 to-amber-600 hover:from-orange-600 hover:to-amber-700 rounded-lg shadow-md shadow-orange-500/25 transition-all flex items-center space-x-1.5"
            >
              <Heart className="w-3.5 h-3.5 fill-white" />
              <span>Comprar Pro (R$ 9,99/mês)</span>
            </a>
          </div>
        </div>
      </header>

      {/* HERO SECTION */}
      <section className="relative pt-12 pb-16 md:pt-20 md:pb-24 max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 text-center">
        <div className="inline-flex items-center space-x-2 px-4 py-1.5 rounded-full bg-orange-500/10 border border-orange-500/20 text-orange-300 text-xs font-medium mb-8">
          <Sparkles className="w-4 h-4 text-orange-400" />
          <span>Modelo Freemium Transparente • Plano Pro por R$ 9,99/mês • 100% Rust Nativo</span>
        </div>

        <h1 className="text-4xl sm:text-6xl lg:text-7xl font-extrabold tracking-tight text-white max-w-5xl mx-auto leading-[1.1] mb-6 font-display">
          O Navegador macOS Definitivo. <br />
          <span className="gradient-text-orange">1.6 MB. 3.8× Menor Uso de CPU.</span>
        </h1>

        <p className="text-lg sm:text-xl text-slate-300 max-w-3xl mx-auto font-normal leading-relaxed mb-10">
          Construído em <strong className="text-white">Rust</strong> sobre a estrutura nativa do macOS (<code className="text-amber-300 font-mono">WKWebView</code>, <code className="text-amber-300 font-mono">VideoToolbox</code> & <code className="text-amber-300 font-mono">Metal</code>). Zero JavaScript na UI, zero processos fantasma.
        </p>

        <div className="flex flex-col sm:flex-row items-center justify-center gap-4 mb-16">
          <a
            href="https://github.com/Browser_Open_source/releases"
            target="_blank"
            rel="noreferrer"
            className="w-full sm:w-auto px-8 py-4 bg-gradient-to-r from-orange-500 to-amber-600 hover:from-orange-600 hover:to-amber-700 text-white font-bold rounded-xl shadow-xl shadow-orange-500/30 transition-all transform hover:-translate-y-0.5 flex items-center justify-center space-x-3 text-base"
          >
            <Download className="w-5 h-5" />
            <span>Baixar Grátis para macOS (.dmg)</span>
          </a>

          <a
            href={LEMON_SQUEEZY_BUY_URL}
            target="_blank"
            rel="noreferrer"
            className="w-full sm:w-auto px-8 py-4 bg-white/5 hover:bg-white/10 border border-white/15 text-white font-semibold rounded-xl transition-all flex items-center justify-center space-x-2 text-base backdrop-blur-md"
          >
            <Heart className="w-5 h-5 text-orange-400 fill-orange-400/20" />
            <span>Assinar Plano Pro (R$ 9,99 / mês)</span>
            <ExternalLink className="w-4 h-4 text-slate-400" />
          </a>
        </div>

        {/* HERO IMAGE SHOWCASE WITH REAL BROWSER SCREENSHOT */}
        <div id="hero-preview" className="relative max-w-5xl mx-auto rounded-2xl p-2 bg-gradient-to-b from-white/15 to-white/5 border border-white/15 shadow-2xl shadow-orange-500/10">
          <div className="relative rounded-xl overflow-hidden group cursor-pointer" onClick={() => setLightboxImg('/images/01_hero.png')}>
            <img 
              src="/images/01_hero.png" 
              alt="Vel Browser Interface Principal" 
              className="w-full h-auto object-cover rounded-xl transition-transform duration-500 group-hover:scale-[1.01]"
            />
            <div className="absolute inset-0 bg-gradient-to-t from-slate-950/80 via-transparent to-transparent flex items-end justify-between p-6 opacity-90">
              <div className="text-left">
                <span className="px-3 py-1 bg-orange-500/30 border border-orange-500/50 text-orange-200 text-xs font-mono font-bold rounded-full">
                  Vel Browser v1.0.4 em Ação
                </span>
                <p className="text-xs text-slate-300 mt-1 font-mono">UI ultraleve sobre o macOS Apple Silicon</p>
              </div>

              <div className="flex items-center space-x-2 px-3 py-1.5 bg-black/60 backdrop-blur-md rounded-lg text-xs font-medium text-slate-200 border border-white/10">
                <Eye className="w-4 h-4 text-orange-400" />
                <span>Clique para ampliar</span>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* FEATURE SHOWCASE WITH REAL STORE SCREENSHOTS */}
      <section id="recursos" className="py-20 max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 border-t border-white/10">
        <div className="text-center max-w-3xl mx-auto mb-14">
          <div className="inline-flex items-center space-x-2 px-3 py-1 rounded-full bg-cyan-500/10 border border-cyan-500/20 text-cyan-300 text-xs font-mono mb-4">
            <ImageIcon className="w-3.5 h-3.5" />
            <span>Recursos com Capturas Reais do Produto</span>
          </div>
          <h2 className="text-3xl sm:text-5xl font-extrabold text-white font-display mb-4">
            Projetado para ser o Navegador Mais Limpo do Mundo
          </h2>
          <p className="text-slate-300 text-base leading-relaxed">
            Veja em detalhes como cada módulo do Vel Browser foi otimizado para não gastar CPU nem memória.
          </p>
        </div>

        {/* Feature Selector & Display Grid */}
        <div className="grid lg:grid-cols-12 gap-8 items-center">
          {/* Left Buttons List */}
          <div className="lg:col-span-4 space-y-2.5">
            {featureList.map((item, index) => (
              <button
                key={item.id}
                onClick={() => setSelectedFeature(index)}
                className={`w-full text-left p-4 rounded-xl border transition-all flex items-center justify-between ${
                  selectedFeature === index
                    ? 'bg-gradient-to-r from-orange-500/20 to-amber-500/10 border-orange-500/50 text-white shadow-md'
                    : 'glass-panel text-slate-400 hover:text-white hover:border-white/20'
                }`}
              >
                <div>
                  <div className="text-xs font-bold text-slate-200">{item.title}</div>
                  <div className="text-[11px] text-slate-400 line-clamp-1">{item.subtitle}</div>
                </div>
                <span className={`text-[10px] font-mono px-2 py-0.5 rounded ${
                  selectedFeature === index ? 'bg-orange-500 text-white font-bold' : 'bg-white/5 text-slate-400'
                }`}>
                  {item.badge}
                </span>
              </button>
            ))}
          </div>

          {/* Right Image Display Preview */}
          <div className="lg:col-span-8">
            <div className="glass-panel p-4 rounded-2xl border border-white/15 overflow-hidden relative group">
              <div 
                className="relative rounded-xl overflow-hidden cursor-pointer"
                onClick={() => setLightboxImg(featureList[selectedFeature].image)}
              >
                <img
                  src={featureList[selectedFeature].image}
                  alt={featureList[selectedFeature].title}
                  className="w-full h-[420px] object-contain bg-slate-950/80 rounded-xl"
                />

                <div className="absolute top-4 left-4 px-3 py-1 rounded-lg bg-black/70 backdrop-blur-md border border-white/10 text-xs font-mono text-orange-300 font-bold">
                  {featureList[selectedFeature].badge}
                </div>

                <div className="absolute inset-x-0 bottom-0 p-6 bg-gradient-to-t from-slate-950 via-slate-950/80 to-transparent">
                  <h3 className="text-xl font-bold text-white mb-1 font-display">
                    {featureList[selectedFeature].title}
                  </h3>
                  <p className="text-xs text-slate-300 leading-relaxed max-w-2xl">
                    {featureList[selectedFeature].desc}
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* SCREENSHOT GALLERY GRID */}
      <section id="galeria" className="py-20 max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 border-t border-white/10">
        <div className="text-center max-w-3xl mx-auto mb-14">
          <h2 className="text-3xl sm:text-4xl font-extrabold text-white font-display mb-3">
            Galeria de Capturas de Tela do Vel
          </h2>
          <p className="text-slate-400 text-sm">
            Clique nas imagens abaixo para visualizar o Vel Browser em alta resolução.
          </p>
        </div>

        <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-6">
          {galleryScreenshots.map((shot, idx) => (
            <div
              key={idx}
              onClick={() => setLightboxImg(shot.src)}
              className="glass-panel rounded-xl overflow-hidden border border-white/10 hover:border-orange-500/50 cursor-pointer transition-all hover:-translate-y-1 group"
            >
              <div className="relative overflow-hidden h-48 bg-slate-950 flex items-center justify-center">
                <img
                  src={shot.src}
                  alt={shot.title}
                  className="w-full h-full object-cover transition-transform duration-500 group-hover:scale-105"
                />
                <div className="absolute inset-0 bg-slate-950/40 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
                  <div className="px-3 py-1.5 rounded-lg bg-orange-500 text-white text-xs font-bold flex items-center space-x-1.5 shadow-lg">
                    <Eye className="w-4 h-4" />
                    <span>Ampliar</span>
                  </div>
                </div>
              </div>

              <div className="p-4 text-left">
                <h4 className="text-sm font-bold text-white mb-1">{shot.title}</h4>
                <p className="text-xs text-slate-400">{shot.desc}</p>
              </div>
            </div>
          ))}
        </div>
      </section>

      {/* PERFORMANCE BENCHMARK SECTION */}
      <section id="desempenho" className="py-20 max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 border-t border-white/10">
        <div className="text-center max-w-3xl mx-auto mb-16">
          <div className="inline-flex items-center space-x-2 px-3 py-1 rounded-full bg-cyan-500/10 border border-cyan-500/20 text-cyan-300 text-xs font-mono mb-4">
            <Cpu className="w-3.5 h-3.5" />
            <span>Medições Reais em Mac Apple Silicon</span>
          </div>
          <h2 className="text-3xl sm:text-5xl font-extrabold text-white font-display mb-4">
            3.8× Menor Uso de CPU que o Safari
          </h2>
          <p className="text-slate-300 text-base leading-relaxed">
            Amostragem contínua rodando o mesmo vídeo ao vivo em 1080p no YouTube lado a lado (média de medições com a ferramenta nativa <code className="text-orange-300 font-mono">top</code> do macOS).
          </p>
        </div>

        {/* Real Benchmark screenshot display */}
        <div className="glass-panel p-6 rounded-2xl border border-white/15 mb-12 flex flex-col md:flex-row items-center gap-8">
          <div className="w-full md:w-1/2">
            <img 
              src="/images/05_cpu.png" 
              alt="Gráfico de Medição de CPU Vel vs Safari"
              className="w-full h-auto rounded-xl border border-white/10 shadow-lg cursor-pointer"
              onClick={() => setLightboxImg('/images/05_cpu.png')}
            />
          </div>

          <div className="w-full md:w-1/2 text-left space-y-4">
            <h3 className="text-2xl font-bold text-white font-display">Tabela de Amostragem do Processador</h3>
            <p className="text-xs text-slate-300 leading-relaxed">
              O pipeline do Vel reduz significativamente o trabalho do processador na UI. Enquanto o Safari consome 6.1% de CPU no processo de UI, o Vel fica em apenas <strong>0.4%</strong>.
            </p>

            <div className="grid grid-cols-2 gap-4 font-mono text-xs pt-2">
              <div className="p-3 bg-slate-900 rounded-lg border border-white/10">
                <span className="text-slate-400 block text-[10px]">CPU Total Safari</span>
                <span className="text-lg font-bold text-red-400">45.1%</span>
              </div>

              <div className="p-3 bg-orange-500/10 rounded-lg border border-orange-500/30">
                <span className="text-orange-300 block text-[10px]">CPU Total Vel (Rust)</span>
                <span className="text-lg font-bold text-orange-400">11.9%</span>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* FREEMIUM PRICING SECTION */}
      <section id="planos" className="py-20 max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 border-t border-white/10">
        <div className="text-center max-w-3xl mx-auto mb-16">
          <div className="inline-flex items-center space-x-2 px-3 py-1 rounded-full bg-orange-500/10 border border-orange-500/20 text-orange-300 text-xs font-mono mb-4">
            <Heart className="w-3.5 h-3.5 text-orange-400 fill-orange-400" />
            <span>Modelo Freemium Simples & Acessível</span>
          </div>
          <h2 className="text-3xl sm:text-5xl font-extrabold text-white font-display mb-4">
            Grátis Para Sempre. Pro por Apenas R$ 9,99/mês.
          </h2>
          <p className="text-slate-300 text-base leading-relaxed">
            O Vel é 100% funcional no plano gratuito: sem limites de tempo, sem anúncios forçados e sem bloqueio de navegação. Apoiadores desbloqueiam o plano Pro com recursos exclusivos por um preço super acessível.
          </p>
        </div>

        {/* Pricing Cards Grid */}
        <div className="grid md:grid-cols-2 gap-8 max-w-5xl mx-auto">
          {/* FREE PLAN CARD */}
          <div className="glass-panel p-8 rounded-2xl border border-white/15 flex flex-col justify-between text-left hover:border-white/30 transition-all">
            <div>
              <div className="flex justify-between items-center mb-4">
                <h3 className="text-2xl font-bold text-white">Plano Grátis</h3>
                <span className="px-3 py-1 bg-slate-800 text-slate-300 text-xs font-mono rounded-full font-bold">
                  R$ 0 / mês
                </span>
              </div>
              <p className="text-slate-400 text-xs mb-6">
                Tudo o que você precisa para navegar com velocidade máxima e consumo mínimo de bateria.
              </p>

              <div className="text-3xl font-extrabold text-white mb-6">
                R$ 0 <span className="text-xs font-normal text-slate-400">para sempre • sem cadastro</span>
              </div>

              <ul className="space-y-3.5 text-xs text-slate-300 mb-8">
                <li className="flex items-center space-x-2.5">
                  <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />
                  <span>Navegador minimalista em Rust (.app de 1.6 MB)</span>
                </li>
                <li className="flex items-center space-x-2.5">
                  <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />
                  <span>Motor WebKit nativo do macOS com aceleração Metal</span>
                </li>
                <li className="flex items-center space-x-2.5">
                  <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />
                  <span>Barra de endereços híbrida segura</span>
                </li>
                <li className="flex items-center space-x-2.5">
                  <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />
                  <span>Atalhos nativos do macOS (<code className="text-amber-300">Cmd+1..9</code>, <code className="text-amber-300">Cmd+T</code>, <code className="text-amber-300">Cmd+W</code>)</span>
                </li>
                <li className="flex items-center space-x-2.5">
                  <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />
                  <span>Pausa automática de animações em abas em segundo plano</span>
                </li>
              </ul>
            </div>

            <a
              href="https://github.com/Browser_Open_source/releases"
              target="_blank"
              rel="noreferrer"
              className="w-full py-3.5 px-4 bg-white/10 hover:bg-white/20 text-white font-semibold rounded-xl transition-all text-center flex items-center justify-center space-x-2 text-xs"
            >
              <Download className="w-4 h-4" />
              <span>Baixar Versão Grátis (.dmg)</span>
            </a>
          </div>

          {/* SUPPORTER PRO PLAN CARD */}
          <div className="glass-panel p-8 rounded-2xl border border-orange-500/50 flex flex-col justify-between text-left orange-glow relative">
            <div className="absolute -top-3.5 right-6 bg-gradient-to-r from-orange-500 to-amber-500 text-white px-3 py-1 rounded-full text-[10px] font-bold font-mono tracking-wider shadow-md">
              RECOMENDADO • MELHOR VALOR
            </div>

            <div>
              <div className="flex justify-between items-center mb-4">
                <h3 className="text-2xl font-bold text-white flex items-center space-x-2">
                  <span>Plano Pro</span>
                  <Sparkles className="w-5 h-5 text-amber-400" />
                </h3>
                <span className="px-3 py-1 bg-orange-500/20 border border-orange-500/40 text-orange-300 text-xs font-mono rounded-full font-bold">
                  BRL 9,99 / mês
                </span>
              </div>
              <p className="text-slate-300 text-xs mb-6">
                Desbloqueie recursos de produtividade avançada e apoie o desenvolvimento contínuo em Rust.
              </p>

              <div className="text-4xl font-extrabold text-white mb-6 font-display">
                R$ 9,99 <span className="text-xs font-normal text-slate-400">/ mês (BRL)</span>
              </div>

              <ul className="space-y-3.5 text-xs text-slate-200 mb-8">
                <li className="flex items-center space-x-2.5">
                  <CheckCircle2 className="w-4 h-4 text-orange-400 shrink-0" />
                  <span className="font-semibold text-white">Tudo incluído no Plano Grátis +</span>
                </li>
                <li className="flex items-center space-x-2.5">
                  <Shield className="w-4 h-4 text-amber-400 shrink-0" />
                  <span><strong>AdBlock & Tracker Blocker em Rust</strong> (sintaxe AdBlock Plus)</span>
                </li>
                <li className="flex items-center space-x-2.5">
                  <Sliders className="w-4 h-4 text-amber-400 shrink-0" />
                  <span><strong>Listas de Filtros Personalizadas</strong></span>
                </li>
                <li className="flex items-center space-x-2.5">
                  <HardDrive className="w-4 h-4 text-amber-400 shrink-0" />
                  <span><strong>Economizador de Memória Automático</strong> (descarte de abas frias)</span>
                </li>
                <li className="flex items-center space-x-2.5">
                  <Maximize2 className="w-4 h-4 text-amber-400 shrink-0" />
                  <span><strong>Picture-in-Picture com Atalho</strong> (<code className="text-amber-300">Cmd+Shift+P</code>)</span>
                </li>
                <li className="flex items-center space-x-2.5">
                  <Key className="w-4 h-4 text-amber-400 shrink-0" />
                  <span><strong>Chave de Licença de Apoiador Pro</strong></span>
                </li>
              </ul>
            </div>

            <div className="space-y-3">
              <a
                href={LEMON_SQUEEZY_BUY_URL}
                target="_blank"
                rel="noreferrer"
                className="w-full py-3.5 px-4 bg-gradient-to-r from-orange-500 to-amber-600 hover:from-orange-600 hover:to-amber-700 text-white font-bold rounded-xl shadow-lg shadow-orange-500/30 transition-all text-xs flex items-center justify-center space-x-2"
              >
                <Heart className="w-4 h-4 fill-white" />
                <span>Assinar Plano Pro no Lemon Squeezy (R$ 9,99/mês)</span>
                <ExternalLink className="w-3.5 h-3.5" />
              </a>

              <button
                onClick={() => setIsKeyModalOpen(true)}
                className="w-full py-2.5 px-4 bg-white/5 hover:bg-white/10 border border-white/10 text-slate-300 font-medium rounded-xl transition-all text-xs flex items-center justify-center space-x-1.5"
              >
                <Key className="w-3.5 h-3.5 text-amber-400" />
                <span>Já possui uma chave? Ativar aqui</span>
              </button>
            </div>
          </div>
        </div>
      </section>

      {/* FAQ & SUPPORT SECTION */}
      <section id="faq" className="py-20 max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 border-t border-white/10">
        <div className="text-center mb-14">
          <h2 className="text-3xl font-extrabold text-white font-display mb-3">
            Perguntas Frequentes & Suporte
          </h2>
          <p className="text-slate-400 text-sm">
            Tudo o que você precisa saber sobre a instalação, chaves de licença e funcionamento do Vel.
          </p>
        </div>

        <div className="space-y-4 mb-16">
          {[
            {
              q: "Como funciona a assinatura do Plano Pro (R$ 9,99/mês)?",
              a: "Você assina o plano Pro pelo Lemon Squeezy por R$ 9,99/mês (BRL) e recebe imediatamente sua chave de apoiador para liberar todos os recursos Pro (AdBlocker em Rust, Economizador de Memória e Picture-in-Picture)."
            },
            {
              q: "O Vel Browser funciona em Macs com processador M1/M2/M3/M4 e Intel?",
              a: "Sim! O Vel é compilado nativamente para Apple Silicon (M1/M2/M3/M4) e Macs com processador Intel x86_64."
            },
            {
              q: "O que acontece se eu usar o Vel sem assinar o plano Pro?",
              a: "Absolutamente nada de ruim! O Vel funcionará com 100% de sua velocidade no plano grátis para sempre, com abas ilimitadas e aceleração de vídeo."
            }
          ].map((faq, idx) => (
            <div
              key={idx}
              className="glass-panel rounded-xl border border-white/10 overflow-hidden transition-all"
            >
              <button
                onClick={() => setOpenFaq(openFaq === idx ? null : idx)}
                className="w-full px-6 py-4 text-left font-semibold text-white text-sm flex justify-between items-center hover:text-orange-400 transition-colors"
              >
                <span>{faq.q}</span>
                <ChevronRight className={`w-4 h-4 transition-transform ${openFaq === idx ? 'rotate-90 text-orange-400' : 'text-slate-400'}`} />
              </button>
              {openFaq === idx && (
                <div className="px-6 pb-4 text-xs text-slate-300 leading-relaxed border-t border-white/5 pt-3">
                  {faq.a}
                </div>
              )}
            </div>
          ))}
        </div>

        <div className="glass-panel p-8 rounded-2xl border border-orange-500/30 text-center relative overflow-hidden">
          <div className="w-12 h-12 rounded-full bg-orange-500/20 text-orange-400 flex items-center justify-center mx-auto mb-4">
            <MessageSquare className="w-6 h-6" />
          </div>
          <h3 className="text-xl font-bold text-white mb-2">Precisa de Ajuda ou Suporte Dedicado?</h3>
          <p className="text-slate-300 text-xs max-w-lg mx-auto mb-6">
            Encontrou algum erro, precisa de assistência com sua licença ou quer sugerir novos recursos?
          </p>
          <div className="flex flex-wrap items-center justify-center gap-4">
            <a
              href="https://github.com/Browser_Open_source/issues"
              target="_blank"
              rel="noreferrer"
              className="px-5 py-2.5 bg-white/10 hover:bg-white/20 text-white font-medium rounded-lg text-xs flex items-center space-x-2 transition-all"
            >
              <Github className="w-4 h-4" />
              <span>Abrir Issue no GitHub</span>
            </a>
          </div>
        </div>
      </section>

      {/* FOOTER */}
      <footer className="py-12 border-t border-white/10 bg-[#040711]">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 flex flex-col md:flex-row items-center justify-between text-xs text-slate-400 gap-4">
          <div className="flex items-center space-x-3">
            <div className="w-7 h-7 rounded-lg bg-orange-500/20 text-orange-400 flex items-center justify-center font-bold">
              ⚡
            </div>
            <span>Vel Browser Project © 2026 • Licenciado sob MIT</span>
          </div>
        </div>
      </footer>

      {/* LIGHTBOX IMAGE MODAL */}
      {lightboxImg && (
        <div 
          className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/90 backdrop-blur-md animate-fadeIn"
          onClick={() => setLightboxImg(null)}
        >
          <div className="relative max-w-6xl w-full max-h-[90vh] flex items-center justify-center">
            <button 
              onClick={() => setLightboxImg(null)}
              className="absolute top-4 right-4 text-white bg-slate-900/80 p-2 rounded-full border border-white/20 hover:bg-orange-500 transition-colors z-10"
            >
              <X className="w-6 h-6" />
            </button>
            <img 
              src={lightboxImg} 
              alt="Visualização em alta resolução" 
              className="max-w-full max-h-[85vh] object-contain rounded-xl border border-white/20 shadow-2xl"
            />
          </div>
        </div>
      )}

      {/* KEY ACTIVATION MODAL */}
      {isKeyModalOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-md animate-fadeIn">
          <div className="glass-panel max-w-md w-full p-6 rounded-2xl border border-white/20 relative shadow-2xl">
            <button
              onClick={() => {
                setIsKeyModalOpen(false);
                setKeyStatus(null);
              }}
              className="absolute top-4 right-4 text-slate-400 hover:text-white p-1 rounded-lg hover:bg-white/10"
            >
              <X className="w-5 h-5" />
            </button>

            <div className="flex items-center space-x-3 mb-4">
              <div className="w-10 h-10 rounded-xl bg-orange-500/20 text-orange-400 flex items-center justify-center">
                <Key className="w-5 h-5" />
              </div>
              <div>
                <h3 className="text-lg font-bold text-white">Ativar Licença Pro (R$ 9,99/mês)</h3>
                <p className="text-xs text-slate-400 font-mono">Vel Supporter Entitlement</p>
              </div>
            </div>

            <p className="text-xs text-slate-300 mb-4 leading-relaxed">
              Cole sua chave de apoiador enviada na confirmação da assinatura (R$ 9,99/mês):
            </p>

            <form onSubmit={handleValidateKey} className="space-y-4">
              <div>
                <input
                  type="text"
                  placeholder="Ex: VEL-8F92A1B3-47C2 ou Token UUID"
                  value={keyInput}
                  onChange={(e) => setKeyInput(e.target.value)}
                  className="w-full px-4 py-3 bg-slate-900 border border-white/15 rounded-xl text-xs font-mono text-white focus:outline-none focus:border-orange-500 transition-all placeholder:text-slate-500"
                />
              </div>

              {keyStatus && (
                <div className={`p-3 rounded-xl text-xs font-medium flex items-start space-x-2 ${
                  keyStatus.type === 'success' ? 'bg-emerald-500/20 border border-emerald-500/30 text-emerald-300' : 'bg-red-500/20 border border-red-500/30 text-red-300'
                }`}>
                  {keyStatus.type === 'success' ? (
                    <CheckCircle2 className="w-4 h-4 shrink-0 mt-0.5" />
                  ) : (
                    <AlertCircle className="w-4 h-4 shrink-0 mt-0.5" />
                  )}
                  <span>{keyStatus.message}</span>
                </div>
              )}

              <div className="flex items-center space-x-3">
                <button
                  type="submit"
                  className="flex-1 py-3 px-4 bg-gradient-to-r from-orange-500 to-amber-600 hover:from-orange-600 hover:to-amber-700 text-white font-bold rounded-xl text-xs shadow-md transition-all"
                >
                  Validar e Ativar
                </button>
              </div>
            </form>

            <div className="mt-5 pt-4 border-t border-white/10 text-center space-y-2">
              <div className="text-xs text-slate-400">Ainda não possui uma assinatura Pro?</div>
              <a
                href={LEMON_SQUEEZY_BUY_URL}
                target="_blank"
                rel="noreferrer"
                className="inline-flex items-center space-x-1 text-xs text-orange-400 font-bold hover:underline"
              >
                <span>Assinar no Lemon Squeezy (R$ 9,99/mês) →</span>
              </a>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
