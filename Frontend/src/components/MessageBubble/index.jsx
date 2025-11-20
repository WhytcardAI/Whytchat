export default function MessageBubble({ role, content }) {
  const isUser = role === "user";
  
  return (
    <div className={`max-w-4xl mx-auto flex w-full ${isUser ? "justify-end" : "justify-start"} group`}>
      <div
        className={`
          relative max-w-[85%] sm:max-w-[75%] rounded-2xl px-5 py-3.5 text-sm leading-7 shadow-sm whitespace-pre-wrap transition-all duration-200
          ${isUser
            ? "bg-gradient-to-br from-primary to-orange-600 text-white rounded-tr-none shadow-lg shadow-primary/20 border border-white/10"
            : "glass text-foreground rounded-tl-none border border-white/5 hover:bg-white/5 hover:border-white/10"}
        `}
      >
        {/* Petit indicateur décoratif pour l'IA */}
        {!isUser && (
            <div className="absolute -left-2 -top-2 w-4 h-4 rounded-full bg-white/5 backdrop-blur-md border border-white/10 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity duration-300">
                <div className="w-1.5 h-1.5 bg-primary rounded-full"></div>
            </div>
        )}
        
        <div className={isUser ? "text-white/95 font-light" : "text-gray-100/90"}>
          {content}
        </div>
      </div>
    </div>
  );
}
