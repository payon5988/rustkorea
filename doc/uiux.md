# UIUX: Rust Korea Hub 디자인 시스템 (현행)

초기 기획은 "다크 우선 + 앰버 포인트(#DE5B30)"였다. 현재 사이트는 **라이트 기본 + 블루 포인트**이고, 다크는 사용자가 토글해서 쓴다.
앰버(#DE5B30)는 PWA 매니페스트 테마색과 일부 섹션 라벨에만 남아 있다.
토큰의 단일 출처는 `src/app.rs`의 `:root` 블록이고, 관리자 콘솔은 `src/admin_ui.rs`에 따로 있다.

## 1. 디자인 토큰 (`src/app.rs`)

```css
:root {
  --font-sans: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Noto Sans KR', system-ui, sans-serif;
  --color-brand: #2563EB;        /* 링크·주요 버튼·활성 탭 */
  --color-brand-hover: #1D4ED8;
  --color-bg: #FFFFFF;
  --color-bg-secondary: #f8f9fa;
  --color-border: #E4E4E7;
  --color-text: #16181d;
  --color-text-secondary: #6b7280;
  --color-text-muted: #9ca3af;
}
html.dark body  { background: #0f1117; color: #e5e7eb; }
html.dark .card { background: #1a1d27; border-color: #2a2d3a; }
```

- **테마**: `theme` 쿠키(light/dark)와 `toggleTheme()`. 관리자 콘솔은 시스템/라이트/다크 3단계다.
- **본문**: 16px, `line-height: 1.6`. 컨테이너 최대 폭은 1280px(일반)과 960px(좁은 본문)이다.
- **코드 강조**: highlight.js 11.9.0(cdnjs, 버전 고정 + SRI).
- **에디터**: Toast UI Editor. 다크 테마의 간섭을 막으려고 항상 라이트 표면으로 띄운다.

## 2. 공통 컴포넌트 클래스

| 클래스 | 용도 |
|---|---|
| `.card` | 흰 배경, 1px 테두리, 반경 8px, hover 시 그림자 |
| `.btn-primary` / `.btn-secondary` | 주요 버튼(브랜드 블루) / 보조 버튼(테두리) |
| `.badge` | 알약형 라벨(12px) |
| `.input` | 16px 입력(모바일 확대 방지), focus 시 브랜드 링 |

## 3. 내비게이션

- 상단 메뉴, 푸터(서비스·링크·참여 그룹), 모바일 하단 탭은 모두 `/admin/menus`에서 DB로 관리한다.
- 현재 위치 표시는 `aria-current="page"`로 한다.
- **모바일(768px 이하)**
  - 상단에는 전체 메뉴 버튼(`<dialog>`)이 있다.
  - 하단 탭은 고정 64px에 safe-area를 더하고, 항목이 많으면 가로 스크롤된다. 로그인/프로필 탭은 자동으로 붙는다.

## 4. 콘텐츠 화면

- **매뉴얼 챕터**: 본문과 TOC를 보여 주고, 진행률을 저장한다(`reading_progress`).
- **블로그·상품 이미지**: 자르지 않고 원본 비율로 표시한다(`object-fit: contain`). 상품 상세는 메인 480px에 썸네일 스트립이 붙고, 라이트박스로 확대된다.

## 5. 접근성

- 입력 폰트 16px, 아이콘 버튼 `aria-label`, 대화상자 `aria-haspopup`/`aria-controls`를 쓴다.
- 남은 과제
  - 다크 모드 보조 텍스트(#9ca3af on #0f1117)의 명도 대비를 점검해야 한다.
  - `:focus-visible` 스타일을 통일해야 한다.
