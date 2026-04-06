import { useEffect, useState } from 'react';
import { useKnowledgeBaseStore } from '../stores/useKnowledgeBaseStore';
import { useAppStore } from '../stores/useAppStore';
import { Button, Card, Input } from '../components/ui';
import { open } from '@tauri-apps/plugin-dialog';
import { FileText, Search, Upload, Trash2, FolderOpen, Loader2, Brain } from 'lucide-react';
import { CardSkeleton } from '../components/ui/skeleton';

export default function KnowledgeBase() {
  const { setActiveTab, setAgentDraft } = useAppStore();
  const {
    documents,
    searchResults,
    isLoading,
    error,
    selectedCategory,
    initKnowledgeBase,
    addDocument,
    searchKnowledgeBase,
    deleteDocument,
    listDocuments,
    setSelectedCategory,
    clearError,
  } = useKnowledgeBaseStore();

  const [searchQuery, setSearchQuery] = useState('');
  const [newCategory, setNewCategory] = useState('通用');
  const [isSearchMode, setIsSearchMode] = useState(false);

  useEffect(() => {
    // 初始化知识库 - 使用应用数据目录，避免触发开发模式热重载
    initKnowledgeBase('');  // 空字符串让后端使用默认的应用数据目录
  }, []);

  const handleAddDocument = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: '文档',
            extensions: ['txt', 'docx', 'pdf'],
          },
        ],
      });

      if (selected) {
        await addDocument(selected as string, newCategory);
      }
    } catch (err) {
      console.error('添加文档失败:', err);
    }
  };

  const handleSearch = async () => {
    if (searchQuery.trim()) {
      setIsSearchMode(true);
      await searchKnowledgeBase(searchQuery);
    }
  };

  const handleClearSearch = () => {
    setIsSearchMode(false);
    setSearchQuery('');
    listDocuments();
  };

  const handleSendToAgent = (doc: { name: string; category: string; content: string }) => {
    setAgentDraft({
      task: '请基于这份知识资料，生成一个适合抖音口播的视频脚本，并补充 3 个可直接开拍的选题角度。',
      transcript: doc.content,
      referenceText: `资料名称：${doc.name}\n资料分类：${doc.category}`,
      preferredSkillId: 'knowledge-grounded-script',
      useKnowledgeBase: true,
    });
    setActiveTab('agent-studio');
  };

  const categories = Array.from(new Set(documents.map((doc) => doc.category)));

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">知识库管理</h1>
        <div className="flex gap-2">
          <Button onClick={handleAddDocument} disabled={isLoading}>
            <Upload className="w-4 h-4 mr-2" />
            上传文档
          </Button>
        </div>
      </div>

      {error && (
        <Card className="p-4 bg-red-50 border-red-200">
          <p className="text-red-600">{error}</p>
          <Button onClick={clearError} className="mt-2">
            关闭
          </Button>
        </Card>
      )}

      <Card className="p-4">
        <div className="flex gap-2 mb-4">
          <Input
            placeholder="输入分类名称"
            value={newCategory}
            onChange={(e) => setNewCategory(e.target.value)}
            className="w-48"
          />
        </div>

        <div className="flex gap-2">
          <Input
            placeholder="搜索知识库..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            onKeyPress={(e) => e.key === 'Enter' && handleSearch()}
            className="flex-1"
          />
          <Button onClick={handleSearch} disabled={isLoading}>
            {isLoading ? (
              <Loader2 className="w-4 h-4 mr-2 animate-spin" />
            ) : (
              <Search className="w-4 h-4 mr-2" />
            )}
            搜索
          </Button>
          {isSearchMode && (
            <Button onClick={handleClearSearch}>清除搜索</Button>
          )}
        </div>
      </Card>

      <div className="flex gap-2 flex-wrap">
        <Button
          variant={selectedCategory === null ? 'default' : 'outline'}
          onClick={() => setSelectedCategory(null)}
        >
          全部
        </Button>
        {categories.map((category) => (
          <Button
            key={category}
            variant={selectedCategory === category ? 'default' : 'outline'}
            onClick={() => setSelectedCategory(category)}
          >
            {category}
          </Button>
        ))}
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {isLoading ? (
          // 加载骨架屏
          Array.from({ length: 6 }).map((_, i) => (
            <CardSkeleton key={i} />
          ))
        ) : (
          (isSearchMode ? searchResults.map((r) => r.document) : documents).map((doc) => (
            <Card key={doc.id} className="p-4">
              <div className="flex items-start justify-between mb-2">
                <div className="flex items-center gap-2">
                  <FileText className="w-5 h-5 text-blue-500" />
                  <h3 className="font-semibold truncate">{doc.name}</h3>
                </div>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => deleteDocument(doc.id)}
                >
                  <Trash2 className="w-4 h-4 text-red-500" />
                </Button>
              </div>
              <p className="text-sm text-gray-500 mb-2">{doc.category}</p>
              <p className="text-xs text-gray-400 line-clamp-3">
                {doc.content.substring(0, 150)}...
              </p>
              <div className="mt-3 flex items-center justify-between gap-2">
                <p className="text-xs text-gray-400">
                  {new Date(doc.created_at).toLocaleString('zh-CN')}
                </p>
                <Button
                  variant="outline"
                  size="sm"
                  className="h-8 text-xs"
                  onClick={() => handleSendToAgent(doc)}
                >
                  <Brain className="w-3.5 h-3.5 mr-1.5" />
                  发送到 Agent
                </Button>
              </div>
            </Card>
          ))
        )}
      </div>

      {documents.length === 0 && !isLoading && (
        <div className="text-center py-12 text-gray-500">
          <FolderOpen className="w-16 h-16 mx-auto mb-4 opacity-50" />
          <p>暂无文档，点击上传按钮添加文档到知识库</p>
        </div>
      )}
    </div>
  );
}
