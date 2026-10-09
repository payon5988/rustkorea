
---

## 📄 3. UIUX.md (사용자 경험 및 디자인 시스템) - **극대화된 심화판**

```markdown
# UIUX: Rust Korea Hub 디자인 시스템 및 경험 설계

## 1. 디자인 철학: "Midnight Rust & Warm Amber"
개발자 도구의 차가움(Rust의 신뢰성)과 사용자가 추구하는 "따뜻하고 풍부한(Warm and rich)" 감성을 결합.
- **Dark Mode First**: 장시간 코딩/읽기에 최적화된 깊은 다크 테마를 기본으로 하되, 차갑지 않은 따뜻한 앰버(Amber) 톤으로 포인트를 줌.
- **Cinematic Readability**: 시네마틱한 시각적 스토리텔링 원칙을 차용하여, 매뉴얼의 히어로 섹션에는 분위기 있는 그라데이션과 부드러운 그림자 사용.

## 2. 디자인 토큰 (Design Tokens)
### 2.1 컬러 팔레트 (Tailwind CSS 확장)
```css
:root {
  /* 베이스: 따뜻하고 깊은 다크 */
  --color-bg-primary: #121214;      /* 완전 블랙이 아닌 따뜻한 짙은 회색 */
  --color-bg-secondary: #1C1B1A;    /* 카드/사이드바 배경 */
  --color-bg-tertiary: #272624;     /* 호버/인풋 배경 */
  
  /* 포인트: Rust 공식 오렌지의 따뜻한 변형 */
  --color-brand-primary: #DE5B30;   /* 메인 액션, 링크 (기존 #CE422B보다 약간 밝고 따뜻하게) */
  --color-brand-glow: rgba(222, 91, 48, 0.15); /* 그림자/글로우 효과용 */
  
  /* 텍스트: 가독성 최적화 */
  --color-text-primary: #F5F1EA;    /* 순백(#FFF)보다 눈이 편안한 웜 화이트 */
  --color-text-secondary: #A8A29E;  /* 보조 텍스트 (Warm Gray 400) */
  --color-text-muted: #78716C;      /* 비활성/메타 정보 */
  
  /* 상태 */
  --color-success: #10B981;
  --color-warning: #F59E0B;
  --color-error: #EF4444;
}

2.2 타이포그래피 및 간격
Font Family:
한국어 본문: Pretendard (Variable, 400~700)
영어/코드: JetBrains Mono (Ligatures 활성화)
Type Scale: text-xs(0.75rem) ~ text-5xl(3rem). 매뉴얼 본문은 text-base(1rem)에 leading-relaxed(1.625) 적용.
Spacing: 4px 그리드 시스템 기반 (0.25rem 단위).
3. 핵심 화면 및 컴포넌트 명세
3.1 매뉴얼 리딩 화면 (The Reading Experience)
레이아웃: 3-Pane 구조 (좌: 목차 250px, 중앙: 콘텐츠 최대 800px, 우: 진행률/TOC 200px). 모바일에서는 목차가 드로어(Drawer)로 숨김.
비율 최적화: 매뉴얼 내 삽입되는 YouTube Shorts나 프로모션 영상은 사용자가 선호하는 9:16 (세로) 또는 4:5 (피드) 종횡비를 명시적으로 지원하도록 컨테이너 설계 (aspect-ratio: 9/16).
마이크로 인터랙션:
코드 블록 호버 시: 우측 상단에 복사 아이콘이 부드럽게(transition-opacity duration-200) 페이드인.
스크롤 진행률: 우측 상단에 얇은 --color-brand-primary 프로그레스 바가 스크롤에 연동되어 채워짐.
3.2 Topcoat UI 컴포넌트 매핑
topcoat ui add로 가져온 컴포넌트를 다음과 같이 커스터마이징:
Button: bg-brand-primary hover:bg-brand-primary/90 text-white rounded-md shadow-[0_0_15px_var(--color-brand-glow)] transition-all
Card: bg-bg-secondary border border-white/5 rounded-xl p-6 hover:border-brand-primary/30 transition-colors
4. 접근성 (Accessibility, WCAG 2.1 AA)
모든 색상 조합의 명도 대비(Contrast Ratio) 4.5:1 이상 보장.
포커스 상태(:focus-visible): outline-2 outline-brand-primary outline-offset-2 적용.
이미지/아이콘: 모든 img 태그에 의미 있는 alt 텍스트 강제.
