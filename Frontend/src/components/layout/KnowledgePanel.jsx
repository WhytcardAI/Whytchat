import { useState } from "react";
import { useTranslation } from "react-i18next";
import { open } from "@tauri-apps/plugin-dialog";
import { api } from "../../lib/api";
import { useKnowledgeStore } from "../../stores/knowledgeStore";
import { Database, FileText, Plus, Trash2, Loader2, CheckCircle2 } from "lucide-react";

export default function KnowledgePanel() {
  const { documents, addDocument, isIndexing, setIndexing, removeDocument } = useKnowledgeStore();
  const { t } = useTranslation();
  const [error, setError] = useState(null);

  const handleAddDocument = async () => {
    try {
      const selectedPath = await open({
        multiple: false,
        filters: [{ name: "Documents", extensions: ["pdf", "txt", "md"] }],
      });

      if (selectedPath && typeof selectedPath === "string") {
        setIndexing(true);
        setError(null);

        // Appel API Backend pour ingestion
        const result = await api.ingestFile(selectedPath);
        console.log("Ingestion result:", result);

        // Mise à jour du store
        addDocument({
          name: selectedPath.split(/[\\/]/).pop(),
          path: selectedPath,
          type: selectedPath.split(".").pop(),
        });

        setIndexing(false);
      }
    } catch (err) {
      console.error("Failed to add document:", err);
      setError(err.message || t("chat.knowledge.error_add"));
      setIndexing(false);
    }
  };

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2 font-medium text-sm text-foreground/80">
          <Database className="w-4 h-4 text-primary" />
          <span>{t("chat.knowledge.title")}</span>
        </div>
        <button
          onClick={handleAddDocument}
          disabled={isIndexing}
          className="p-1.5 rounded-md bg-secondary hover:bg-secondary/80 text-muted-foreground hover:text-foreground transition-colors disabled:opacity-50"
          title={t("chat.knowledge.add_doc")}
        >
          {isIndexing ? <Loader2 className="w-4 h-4 animate-spin" /> : <Plus className="w-4 h-4" />}
        </button>
      </div>

      {error && (
        <div className="mb-3 p-2 text-xs text-red-400 bg-red-400/10 rounded border border-red-400/20">
          {error}
        </div>
      )}

      <div className="flex-1 overflow-y-auto space-y-2 pr-1 scrollbar-hide">
        {documents.length === 0 ? (
          <div className="text-center py-8 text-muted-foreground/40 text-xs">
            <FileText className="w-8 h-8 mx-auto mb-2 opacity-50" />
            <p>{t("chat.knowledge.empty")}</p>
            <p>{t("chat.knowledge.drag_drop")}</p>
          </div>
        ) : (
          documents.map((doc) => (
            <div
              key={doc.id}
              className="group flex items-center justify-between p-2 rounded-lg bg-secondary/30 hover:bg-secondary/50 border border-transparent hover:border-border transition-all"
            >
              <div className="flex items-center gap-3 overflow-hidden">
                <div className="w-8 h-8 rounded bg-background/50 flex items-center justify-center shrink-0">
                  <FileText className="w-4 h-4 text-blue-400" />
                </div>
                <div className="flex flex-col min-w-0">
                  <span
                    className="text-xs font-medium truncate text-foreground/90"
                    title={doc.name}
                  >
                    {doc.name}
                  </span>
                  <span className="text-[10px] text-muted-foreground flex items-center gap-1">
                    {doc.type.toUpperCase()} •{" "}
                    <CheckCircle2 className="w-2.5 h-2.5 text-green-500" />{" "}
                    {t("chat.knowledge.ready")}
                  </span>
                </div>
              </div>
              <button
                onClick={() => removeDocument(doc.id)}
                className="opacity-0 group-hover:opacity-100 p-1.5 text-muted-foreground hover:text-red-400 transition-all"
                title={t("chat.knowledge.delete")}
              >
                <Trash2 className="w-3 h-3" />
              </button>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
