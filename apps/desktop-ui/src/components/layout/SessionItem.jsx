import { useState } from 'react';
import { MessageSquare, Star, MoreVertical, FolderInput, Trash2 } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { useAppStore } from '../../store/appStore';
import { cn } from '../../lib/utils';
import { logger } from '../../lib/logger';

export function SessionItem({ session, active, onClick }) {
  const { t } = useTranslation('common');
  const { toggleFavorite, folders, moveSessionToFolder, deleteSession } = useAppStore();
  const [showMenu, setShowMenu] = useState(false);
  const [showFolderMenu, setShowFolderMenu] = useState(false);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);

  const handleFavorite = async (e) => {
    e.stopPropagation();
    logger.ui.click('SessionItem:Favorite', { sessionId: session.id });
    await toggleFavorite(session.id);
  };

  const handleMenuClick = (e) => {
    e.stopPropagation();
    logger.ui.click('SessionItem:Menu', { sessionId: session.id });
    setShowMenu(!showMenu);
    setShowFolderMenu(false);
  };

  const handleMoveToFolder = async (folderId) => {
    logger.ui.click('SessionItem:MoveToFolder', { sessionId: session.id, folderId });
    await moveSessionToFolder(session.id, folderId);
    setShowMenu(false);
    setShowFolderMenu(false);
  };

  const handleDelete = async () => {
    logger.ui.click('SessionItem:Delete', { sessionId: session.id });
    await deleteSession(session.id);
    setShowMenu(false);
    setShowDeleteConfirm(false);
  };

  const displayName = session.title || 'Session ' + session.id.slice(-8);

  return (
    <div className="relative group px-2">
      <div
        role="button"
        tabIndex={0}
        onClick={onClick}
        onKeyDown={(e) => {
          if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            onClick(e);
          }
        }}
        className={cn(
          'w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm transition-all text-left border border-transparent cursor-pointer select-none',
          active
            ? 'bg-surface text-primary border-border shadow-sm'
            : 'text-muted-foreground hover:bg-surface/50 hover:text-foreground'
        )}
      >
        <MessageSquare size={16} className={cn("shrink-0", active ? "text-primary" : "text-muted-foreground/50")} />

        <span className="truncate flex-1 font-medium">{displayName}</span>

        {/* Favorite star */}
        <button
          onClick={handleFavorite}
          className={cn(
            'p-1 rounded transition-all',
            session.is_favorite
              ? 'text-yellow-500 hover:text-yellow-600'
              : 'text-muted/20 hover:text-yellow-500 opacity-0 group-hover:opacity-100'
          )}
          title={session.is_favorite ? t('sessions.unfavorite', 'Remove from favorites') : t('sessions.favorite', 'Add to favorites')}
        >
          <Star size={14} fill={session.is_favorite ? 'currentColor' : 'none'} />
        </button>

        {/* More menu */}
        <button
          onClick={handleMenuClick}
          className="p-1 rounded text-muted/20 hover:text-foreground opacity-0 group-hover:opacity-100 transition-all"
        >
          <MoreVertical size={14} />
        </button>
      </div>

      {/* Context Menu */}
      {showMenu && (
        <div className="absolute right-0 top-full mt-1 w-48 bg-surface border border-border rounded-lg shadow-xl z-50 py-1 animate-fade-in">
          <button
            onClick={(e) => {
              e.stopPropagation();
              setShowFolderMenu(!showFolderMenu);
            }}
            className="w-full flex items-center gap-2 px-3 py-2 text-sm text-text hover:bg-background transition-colors"
          >
            <FolderInput size={14} />
            {t('sessions.move_to_folder', 'Move to folder')}
          </button>

          {showFolderMenu && (
            <div className="border-t border-border mt-1 pt-1">
              <button
                onClick={() => handleMoveToFolder(null)}
                className={cn(
                  "w-full flex items-center gap-2 px-3 py-1.5 text-xs hover:bg-background transition-colors",
                  !session.folder_id ? "text-primary" : "text-muted"
                )}
              >
                {t('sessions.no_folder', 'No folder')}
              </button>
              {folders.map((folder) => (
                <button
                  key={folder.id}
                  onClick={() => handleMoveToFolder(folder.id)}
                  className={cn(
                    "w-full flex items-center gap-2 px-3 py-1.5 text-xs hover:bg-background transition-colors",
                    session.folder_id === folder.id ? "text-primary" : "text-text"
                  )}
                >
                  <span
                    className="w-2 h-2 rounded-full"
                    style={{ backgroundColor: folder.color }}
                  />
                  {folder.name}
                </button>
              ))}
            </div>
          )}

          {/* Delete button */}
          {!showDeleteConfirm ? (
            <button
              onClick={(e) => {
                e.stopPropagation();
                setShowDeleteConfirm(true);
              }}
              className="w-full flex items-center gap-2 px-3 py-2 text-sm text-red-400 hover:bg-red-500/10 transition-colors border-t border-border mt-1"
            >
              <Trash2 size={14} />
              {t('sessions.delete', 'Delete')}
            </button>
          ) : (
            <div className="border-t border-border mt-1 pt-1 px-3 py-2">
              <p className="text-xs text-muted mb-2">{t('sessions.delete_confirm', 'Delete this session?')}</p>
              <div className="flex gap-2">
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    setShowDeleteConfirm(false);
                  }}
                  className="flex-1 px-2 py-1 text-xs bg-background rounded hover:bg-surface transition-colors"
                >
                  {t('common.cancel', 'Cancel')}
                </button>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    handleDelete();
                  }}
                  className="flex-1 px-2 py-1 text-xs bg-red-500 text-white rounded hover:bg-red-600 transition-colors"
                >
                  {t('common.delete', 'Delete')}
                </button>
              </div>
            </div>
          )}
        </div>
      )}

      {/* Click outside to close menu */}
      {showMenu && (
        <div
          className="fixed inset-0 z-40"
          onClick={() => { setShowMenu(false); setShowFolderMenu(false); }}
        />
      )}
    </div>
  );
}
