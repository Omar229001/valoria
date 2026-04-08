import { useState, useEffect } from "react";

// ── Config API ────────────────────────────────────────────────────────────────
const API = (import.meta as any).env?.VITE_API_URL ?? "http://localhost:8080";

// ── Types ─────────────────────────────────────────────────────────────────────
interface User {
  id: number;
  name: string;
  email: string;
  role: string;
}

interface MarketStats {
  listings_count: number;
  median_price: number | null;
  min_price: number | null;
  max_price: number | null;
}

interface CotationResult {
  cotation_id: number;
  brand: string;
  model: string;
  year: number;
  mileage: number;
  condition: string;
  ml_estimated_price: number;
  market: MarketStats;
  estimated_price: number;
  min_price: number;
  max_price: number;
  confidence: number;
  method: string;
  created_at: string;
}

// ── Styles ────────────────────────────────────────────────────────────────────
const S = {
  page: { minHeight: "100vh", background: "#0f172a", fontFamily: "'Inter', system-ui, sans-serif", color: "#e2e8f0" },
  center: { display: "flex", alignItems: "center", justifyContent: "center", minHeight: "100vh" },
  card: { background: "#1e293b", borderRadius: 16, padding: 40, width: 400, boxShadow: "0 25px 50px rgba(0,0,0,0.5)" },
  logo: { fontSize: 32, fontWeight: 800, color: "#6366f1", marginBottom: 4 },
  subtitle: { fontSize: 14, color: "#64748b", marginBottom: 32 },
  label: { display: "block", fontSize: 13, fontWeight: 600, color: "#94a3b8", marginBottom: 6 },
  input: { width: "100%", padding: "10px 14px", background: "#0f172a", border: "1px solid #334155", borderRadius: 8, color: "#e2e8f0", fontSize: 14, boxSizing: "border-box" as const, outline: "none" },
  btn: { width: "100%", padding: "12px", background: "#6366f1", color: "#fff", border: "none", borderRadius: 8, fontSize: 15, fontWeight: 700, cursor: "pointer", marginTop: 8 },
  btnSecondary: { background: "transparent", color: "#6366f1", border: "1px solid #6366f1", padding: "8px 16px", borderRadius: 8, cursor: "pointer", fontSize: 13, fontWeight: 600 },
  error: { background: "#7f1d1d33", border: "1px solid #ef4444", color: "#fca5a5", padding: "10px 14px", borderRadius: 8, fontSize: 13, marginBottom: 16 },
  header: { background: "#1e293b", padding: "0 32px", display: "flex", alignItems: "center", justifyContent: "space-between", height: 60, borderBottom: "1px solid #334155" },
  main: { maxWidth: 1100, margin: "0 auto", padding: "32px 24px" },
  grid: { display: "grid", gridTemplateColumns: "1fr 1fr", gap: 24 },
  panel: { background: "#1e293b", borderRadius: 12, padding: 28, border: "1px solid #334155" },
  panelTitle: { fontSize: 18, fontWeight: 700, marginBottom: 20, color: "#f1f5f9" },
  select: { width: "100%", padding: "10px 14px", background: "#0f172a", border: "1px solid #334155", borderRadius: 8, color: "#e2e8f0", fontSize: 14 },
  row: { marginBottom: 16 },
  tag: { display: "inline-block", padding: "3px 10px", borderRadius: 99, fontSize: 12, fontWeight: 600 },
  priceBig: { fontSize: 44, fontWeight: 800, color: "#6366f1" },
  priceSmall: { fontSize: 14, color: "#64748b", marginTop: 4 },
  badge: { display: "inline-flex", alignItems: "center", gap: 6, padding: "4px 12px", borderRadius: 99, fontSize: 12, fontWeight: 600, marginRight: 8 },
  tableWrapper: { overflowX: "auto" as const },
  table: { width: "100%", borderCollapse: "collapse" as const, fontSize: 13 },
  th: { padding: "10px 12px", textAlign: "left" as const, color: "#64748b", fontWeight: 600, borderBottom: "1px solid #334155" },
  td: { padding: "10px 12px", borderBottom: "1px solid #1e293b" },
};

// ── Auth Page ──────────────────────────────────────────────────────────────────
function AuthPage({ onLogin }: { onLogin: (token: string, user: User) => void }) {
  const [mode, setMode] = useState<"login" | "register">("login");
  const [form, setForm] = useState({ name: "", email: "", password: "" });
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  const set = (k: string, v: string) => setForm(f => ({ ...f, [k]: v }));

  const submit = async () => {
    setError(""); setLoading(true);
    try {
      const url = mode === "login" ? `${API}/api/auth/login` : `${API}/api/auth/register`;
      const body = mode === "login" ? { email: form.email, password: form.password } : form;
      const res = await fetch(url, { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify(body) });
      const data = await res.json();
      if (!res.ok) throw new Error(data.error ?? "Erreur serveur");
      onLogin(data.token, data.user);
    } catch (e: any) {
      setError(e.message);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div style={S.center}>
      <div style={S.card}>
        <div style={S.logo}>Valoria</div>
        <div style={S.subtitle}>Plateforme de cotation de véhicules d'occasion</div>
        {error && <div style={S.error}>{error}</div>}
        {mode === "register" && (
          <div style={S.row}>
            <label style={S.label}>Nom</label>
            <input style={S.input} value={form.name} onChange={e => set("name", e.target.value)} placeholder="Omar Hadji" />
          </div>
        )}
        <div style={S.row}>
          <label style={S.label}>Email</label>
          <input style={S.input} type="email" value={form.email} onChange={e => set("email", e.target.value)} placeholder="omar@valoria.com" />
        </div>
        <div style={S.row}>
          <label style={S.label}>Mot de passe</label>
          <input style={S.input} type="password" value={form.password} onChange={e => set("password", e.target.value)} placeholder="••••••••" onKeyDown={e => e.key === "Enter" && submit()} />
        </div>
        <button style={S.btn} onClick={submit} disabled={loading}>
          {loading ? "Chargement..." : mode === "login" ? "Se connecter" : "Créer un compte"}
        </button>
        <div style={{ textAlign: "center", marginTop: 20, fontSize: 13, color: "#64748b" }}>
          {mode === "login" ? "Pas encore de compte ? " : "Déjà un compte ? "}
          <span style={{ color: "#6366f1", cursor: "pointer", fontWeight: 600 }} onClick={() => { setMode(mode === "login" ? "register" : "login"); setError(""); }}>
            {mode === "login" ? "S'inscrire" : "Se connecter"}
          </span>
        </div>
      </div>
    </div>
  );
}

// ── Cotation Form ──────────────────────────────────────────────────────────────
function CotationForm({ token, onResult }: { token: string; onResult: (r: CotationResult) => void }) {
  const [form, setForm] = useState({ brand: "", model: "", year: new Date().getFullYear() - 3, mileage: 50000, fuel: "essence", transmission: "manuelle", condition: "bon" });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  const set = (k: string, v: any) => setForm(f => ({ ...f, [k]: v }));

  const submit = async () => {
    if (!form.brand || !form.model) return setError("Marque et modèle requis");
    setError(""); setLoading(true);
    try {
      const res = await fetch(`${API}/api/cotation`, {
        method: "POST",
        headers: { "Content-Type": "application/json", Authorization: `Bearer ${token}` },
        body: JSON.stringify(form),
      });
      const data = await res.json();
      if (!res.ok) throw new Error(data.error ?? data.detail ?? "Erreur serveur");
      onResult(data);
    } catch (e: any) {
      setError(e.message);
    } finally {
      setLoading(false);
    }
  };

  const Field = ({ label, children }: any) => (
    <div style={S.row}><label style={S.label}>{label}</label>{children}</div>
  );

  return (
    <div style={S.panel}>
      <div style={S.panelTitle}>🔍 Nouvelle cotation</div>
      {error && <div style={S.error}>{error}</div>}
      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12 }}>
        <Field label="Marque">
          <input style={S.input} value={form.brand} onChange={e => set("brand", e.target.value)} placeholder="Renault" />
        </Field>
        <Field label="Modèle">
          <input style={S.input} value={form.model} onChange={e => set("model", e.target.value)} placeholder="Clio" />
        </Field>
        <Field label="Année">
          <input style={S.input} type="number" value={form.year} onChange={e => set("year", parseInt(e.target.value))} min={2000} max={2026} />
        </Field>
        <Field label="Kilométrage">
          <input style={S.input} type="number" value={form.mileage} onChange={e => set("mileage", parseInt(e.target.value))} step={1000} />
        </Field>
        <Field label="Carburant">
          <select style={S.select} value={form.fuel} onChange={e => set("fuel", e.target.value)}>
            {["essence", "diesel", "hybride", "électrique", "gpl"].map(f => <option key={f}>{f}</option>)}
          </select>
        </Field>
        <Field label="Transmission">
          <select style={S.select} value={form.transmission} onChange={e => set("transmission", e.target.value)}>
            <option value="manuelle">Manuelle</option>
            <option value="automatique">Automatique</option>
          </select>
        </Field>
      </div>
      <Field label="État du véhicule">
        <div style={{ display: "flex", gap: 8 }}>
          {[["excellent", "#22c55e"], ["bon", "#3b82f6"], ["moyen", "#f59e0b"], ["mauvais", "#ef4444"]].map(([v, c]) => (
            <button key={v} onClick={() => set("condition", v)} style={{ flex: 1, padding: "8px 4px", borderRadius: 8, border: `2px solid ${form.condition === v ? c : "#334155"}`, background: form.condition === v ? c + "22" : "transparent", color: form.condition === v ? c : "#64748b", cursor: "pointer", fontSize: 13, fontWeight: 600 }}>
              {v}
            </button>
          ))}
        </div>
      </Field>
      <button style={{ ...S.btn, marginTop: 16 }} onClick={submit} disabled={loading}>
        {loading ? "⏳ Calcul en cours..." : "Calculer le prix d'achat"}
      </button>
    </div>
  );
}

// ── Cotation Result ────────────────────────────────────────────────────────────
function CotationResult({ result }: { result: CotationResult }) {
  const pct = Math.round(result.confidence * 100);
  const condColor: any = { excellent: "#22c55e", bon: "#3b82f6", moyen: "#f59e0b", mauvais: "#ef4444" };

  return (
    <div style={S.panel}>
      <div style={S.panelTitle}>💰 Résultat de la cotation</div>
      <div style={{ textAlign: "center", marginBottom: 24 }}>
        <div style={{ fontSize: 13, color: "#64748b", marginBottom: 4 }}>Prix d'achat recommandé</div>
        <div style={S.priceBig}>{result.estimated_price.toLocaleString("fr-FR")} €</div>
        <div style={S.priceSmall}>
          Fourchette : {result.min_price.toLocaleString("fr-FR")} € → {result.max_price.toLocaleString("fr-FR")} €
        </div>
      </div>

      {/* Barre de confiance */}
      <div style={{ marginBottom: 20 }}>
        <div style={{ display: "flex", justifyContent: "space-between", fontSize: 12, color: "#64748b", marginBottom: 6 }}>
          <span>Indice de confiance</span><span>{pct}%</span>
        </div>
        <div style={{ background: "#0f172a", borderRadius: 99, height: 6 }}>
          <div style={{ width: `${pct}%`, height: "100%", borderRadius: 99, background: pct > 70 ? "#22c55e" : pct > 40 ? "#f59e0b" : "#ef4444" }} />
        </div>
      </div>

      {/* Badges */}
      <div style={{ marginBottom: 20 }}>
        <span style={{ ...S.badge, background: "#6366f133", color: "#818cf8" }}>
          🤖 ML: {result.ml_estimated_price.toLocaleString("fr-FR")} €
        </span>
        <span style={{ ...S.badge, background: condColor[result.condition] + "22", color: condColor[result.condition] }}>
          État: {result.condition}
        </span>
        <span style={{ ...S.badge, background: "#0f172a", color: "#64748b" }}>
          {result.method === "blend" ? "🔀 Blend" : result.method === "ml_only" ? "🤖 ML pur" : "📊 Marché"}
        </span>
      </div>

      {/* Données marché */}
      <div style={{ background: "#0f172a", borderRadius: 8, padding: 16 }}>
        <div style={{ fontSize: 12, fontWeight: 700, color: "#64748b", marginBottom: 12 }}>DONNÉES MARCHÉ RÉELLES</div>
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr 1fr", gap: 12, textAlign: "center" }}>
          {[
            ["Annonces", result.market.listings_count, ""],
            ["Médiane", result.market.median_price?.toLocaleString("fr-FR") ?? "—", "€"],
            ["Min → Max", result.market.min_price ? `${result.market.min_price.toLocaleString("fr-FR")} → ${result.market.max_price?.toLocaleString("fr-FR")}` : "—", "€"],
          ].map(([label, val, unit]) => (
            <div key={label as string}>
              <div style={{ fontSize: 20, fontWeight: 700, color: "#f1f5f9" }}>{val}{unit ? ` ${unit}` : ""}</div>
              <div style={{ fontSize: 11, color: "#64748b", marginTop: 2 }}>{label}</div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

// ── History ────────────────────────────────────────────────────────────────────
function History({ token }: { token: string }) {
  const [items, setItems] = useState<CotationResult[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetch(`${API}/api/cotation/history`, { headers: { Authorization: `Bearer ${token}` } })
      .then(r => r.json())
      .then(d => { setItems(Array.isArray(d) ? d : []); setLoading(false); })
      .catch(() => setLoading(false));
  }, [token]);

  if (loading) return <div style={{ ...S.panel, color: "#64748b" }}>Chargement de l'historique...</div>;

  return (
    <div style={S.panel}>
      <div style={S.panelTitle}>📋 Historique des cotations</div>
      {items.length === 0 ? (
        <div style={{ color: "#64748b", textAlign: "center", padding: 32 }}>
          Aucune cotation pour l'instant.<br />Faites votre première cotation ci-contre !
        </div>
      ) : (
        <div style={S.tableWrapper}>
          <table style={S.table}>
            <thead>
              <tr>
                {["Véhicule", "Année", "KM", "État", "Prix estimé", "Confiance", "Date"].map(h => (
                  <th key={h} style={S.th}>{h}</th>
                ))}
              </tr>
            </thead>
            <tbody>
              {items.map(item => (
                <tr key={item.cotation_id} style={{ transition: "background .15s" }}>
                  <td style={{ ...S.td, fontWeight: 600, color: "#f1f5f9" }}>{item.brand} {item.model}</td>
                  <td style={S.td}>{item.year}</td>
                  <td style={S.td}>{item.mileage.toLocaleString("fr-FR")} km</td>
                  <td style={S.td}>
                    <span style={{ ...S.tag, background: "#6366f111", color: "#818cf8" }}>{item.condition}</span>
                  </td>
                  <td style={{ ...S.td, fontWeight: 700, color: "#6366f1" }}>{item.estimated_price.toLocaleString("fr-FR")} €</td>
                  <td style={S.td}>
                    <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
                      <div style={{ width: 40, height: 4, background: "#0f172a", borderRadius: 99 }}>
                        <div style={{ width: `${Math.round(item.confidence * 100)}%`, height: "100%", borderRadius: 99, background: item.confidence > 0.7 ? "#22c55e" : "#f59e0b" }} />
                      </div>
                      <span style={{ fontSize: 11, color: "#64748b" }}>{Math.round(item.confidence * 100)}%</span>
                    </div>
                  </td>
                  <td style={{ ...S.td, color: "#64748b", fontSize: 12 }}>
                    {new Date(item.created_at).toLocaleDateString("fr-FR")}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}

// ── Dashboard ──────────────────────────────────────────────────────────────────
function Dashboard({ user, token, onLogout }: { user: User; token: string; onLogout: () => void }) {
  const [result, setResult] = useState<CotationResult | null>(null);
  const [refreshHistory, setRefreshHistory] = useState(0);

  const handleResult = (r: CotationResult) => {
    setResult(r);
    setRefreshHistory(n => n + 1);
  };

  return (
    <div style={S.page}>
      {/* Header */}
      <div style={S.header}>
        <div style={{ fontSize: 22, fontWeight: 800, color: "#6366f1" }}>Valoria</div>
        <div style={{ display: "flex", alignItems: "center", gap: 16 }}>
          <span style={{ fontSize: 13, color: "#94a3b8" }}>👤 {user.name}</span>
          <button style={S.btnSecondary} onClick={onLogout}>Déconnexion</button>
        </div>
      </div>

      {/* Main content */}
      <div style={S.main}>
        <div style={{ marginBottom: 24 }}>
          <h1 style={{ fontSize: 24, fontWeight: 800, marginBottom: 4 }}>Cotation de véhicule</h1>
          <p style={{ color: "#64748b", fontSize: 14 }}>
            Entrez les caractéristiques du véhicule pour obtenir son prix d'achat recommandé.
          </p>
        </div>

        {/* Formulaire + Résultat */}
        <div style={S.grid}>
          <CotationForm token={token} onResult={handleResult} />
          {result
            ? <CotationResult result={result} />
            : (
              <div style={{ ...S.panel, display: "flex", alignItems: "center", justifyContent: "center", flexDirection: "column", gap: 12, color: "#334155" }}>
                <div style={{ fontSize: 48 }}>🚗</div>
                <div style={{ fontSize: 15, fontWeight: 600, color: "#475569" }}>Le résultat apparaîtra ici</div>
                <div style={{ fontSize: 13, color: "#334155", textAlign: "center" }}>
                  Remplissez le formulaire et cliquez sur<br />"Calculer le prix d'achat"
                </div>
              </div>
            )
          }
        </div>

        {/* Historique */}
        <div style={{ marginTop: 24 }}>
          <History key={refreshHistory} token={token} />
        </div>
      </div>
    </div>
  );
}

// ── App Root ──────────────────────────────────────────────────────────────────
export default function App() {
  const [token, setToken] = useState<string | null>(() => localStorage.getItem("valoria_token"));
  const [user, setUser] = useState<User | null>(() => {
    try { return JSON.parse(localStorage.getItem("valoria_user") ?? "null"); } catch { return null; }
  });

  const handleLogin = (t: string, u: User) => {
    localStorage.setItem("valoria_token", t);
    localStorage.setItem("valoria_user", JSON.stringify(u));
    setToken(t);
    setUser(u);
  };

  const handleLogout = () => {
    localStorage.removeItem("valoria_token");
    localStorage.removeItem("valoria_user");
    setToken(null);
    setUser(null);
  };

  if (!token || !user) {
    return <div style={S.page}><AuthPage onLogin={handleLogin} /></div>;
  }

  return <Dashboard user={user} token={token} onLogout={handleLogout} />;
}
