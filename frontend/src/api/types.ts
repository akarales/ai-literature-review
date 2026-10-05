export interface Citation {
  pmid: string;
  title: string;
  journal: string;
  year: string;
  relevance: number;
  evidence: string;
}

export interface ReviewResult {
  question: string;
  synthesis: string;
  confidence: string;
  citations: Citation[];
  articles_found: number;
  stub_model: boolean;
  disclaimer: string;
}
