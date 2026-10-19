use std::future::Future;
use topcoat::{
    Result,
    context::Cx,
    router::page,
    view::{View, view},
};

use crate::services::i18n::Locale;

fn page_style() -> &'static str {
    "max-width:780px; margin:0 auto; padding:60px 24px 100px;"
}

fn h1_style() -> &'static str {
    "font-size:1.75rem; font-weight:800; letter-spacing:-0.03em; margin-bottom:8px; color:#16181d;"
}

fn h2_style() -> &'static str {
    "font-size:1.125rem; font-weight:700; margin-top:32px; margin-bottom:12px; color:#16181d;"
}

fn p_style() -> &'static str {
    "font-size:14px; color:#374151; line-height:1.8; margin-bottom:12px;"
}

fn list_style() -> &'static str {
    "font-size:14px; color:#374151; line-height:1.8; margin-bottom:12px; padding-left:20px;"
}

fn note_style() -> &'static str {
    "font-size:13px; color:#92400e; background:#fffbeb; border:1px solid #fde68a; border-radius:8px; padding:10px 14px; margin-bottom:24px;"
}

/// English versions are translations; the Korean text governs.
const EN_NOTICE: &str = "This is an English translation provided for convenience. In case of any discrepancy, the Korean version prevails.";

#[page("/privacy")]
pub async fn privacy_policy(cx: &Cx) -> Result<impl View + '_> {
    let loc = crate::services::i18n::locale(cx);
    let email = crate::config::contact_email();
    Ok(view! {
        <div style=(page_style())>
            if loc == Locale::En {
                <h1 style=(h1_style())>"Privacy Policy"</h1>
                <p style="font-size:13px; color:#9ca3af; margin-bottom:16px;">"Last updated: October 9, 2026"</p>
                <p style=(note_style())>(EN_NOTICE)</p>

                <p style=(p_style())>
                    "Rust Korea Hub (the \"Site\") values the personal information of its users and complies with applicable laws of the Republic of Korea, including the Personal Information Protection Act and the Act on Promotion of Information and Communications Network Utilization and Information Protection."
                </p>

                <h2 style=(h2_style())>"1. Information we collect"</h2>
                <p style=(p_style())>"The Site collects the following information to provide its services."</p>
                <ul style=(list_style())>
                    <li><strong>"When you sign up"</strong>": email address, username, display name"</li>
                    <li><strong>"When you log in with a social account"</strong>": GitHub/Google profile information (email, username, profile image)"</li>
                    <li><strong>"Automatically"</strong>": cookies and usage records (IP address, browser information)"</li>
                </ul>

                <h2 style=(h2_style())>"2. Purposes of use"</h2>
                <ul style=(list_style())>
                    <li>"Identifying and authenticating members"</li>
                    <li>"Providing and operating the service"</li>
                    <li>"Writing and managing community posts"</li>
                    <li>"Delivering announcements"</li>
                    <li>"Improving the service and developing new features"</li>
                </ul>

                <h2 style=(h2_style())>"3. Retention period"</h2>
                <p style=(p_style())>
                    "Personal information is destroyed without delay once the purpose of collection and use has been fulfilled. Where the law requires retention, it is kept for the period prescribed by that law."
                </p>

                <h2 style=(h2_style())>"4. Disclosure to third parties"</h2>
                <p style=(p_style())>
                    "The Site does not, in principle, provide personal information to third parties, except in the following cases:"
                </p>
                <ul style=(list_style())>
                    <li>"When you have given prior consent"</li>
                    <li>"When required by law, or when an investigative agency requests it following the procedures prescribed by law"</li>
                </ul>

                <h2 style=(h2_style())>"5. Cookies"</h2>
                <p style=(p_style())>
                    "The Site uses cookies to improve accessibility and convenience. Cookies are small pieces of data sent by the web server to your browser and stored on your device."
                </p>
                <p style=(p_style())>"Cookies in use:"</p>
                <ul style=(list_style())>
                    <li><strong>"__Host-session"</strong>": keeps you logged in (HttpOnly, SameSite=Lax, Secure, verified server-side)"</li>
                    <li><strong>"csrf_seed"</strong>", "<strong>"oauth_state"</strong>": short-lived protection for login and sign-up forms (about 10 minutes)"</li>
                    <li><strong>"theme"</strong>": dark/light mode preference"</li>
                    <li><strong>"locale"</strong>": language preference"</li>
                    <li><strong>"cookie_consent"</strong>": whether you accepted cookies"</li>
                </ul>
                <p style=(p_style())>
                    "You can refuse cookies in your browser settings, but some features may not work properly."
                </p>

                <h2 style=(h2_style())>"6. Advertising"</h2>
                <p style=(p_style())>
                    "The Site may display ads through Google AdSense. Google uses cookies to serve ads based on prior visits. For details on how Google uses cookies, see "
                    <a href=(crate::util::external_links::GOOGLE_ADS_POLICIES) style="color:#2563EB;">"Google's advertising policies"</a>
                    "."
                </p>
                <p style=(p_style())>
                    "When Google AdSense is enabled, Google, as a third-party vendor, uses cookies and web beacons to serve ads to users on this Site and other sites on the internet."
                </p>

                <h2 style=(h2_style())>"7. Privacy contact"</h2>
                <p style=(p_style())>"For privacy-related inquiries, please contact:"</p>
                <p style=(p_style())>(format!("Email: {email}"))</p>

                <h2 style=(h2_style())>"8. Changes to this policy"</h2>
                <p style=(p_style())>
                    "This policy may be amended to reflect changes in law, policy or security technology. Changes will be announced through the Site's notices."
                </p>
            } else {
                <h1 style=(h1_style())>"개인정보처리방침"</h1>
                <p style="font-size:13px; color:#9ca3af; margin-bottom:32px;">"최종 수정일: 2026년 10월 9일"</p>

                <p style=(p_style())>
                    "Rust Korea Hub('사이트')는 이용자의 개인정보를 중요시하며, 개인정보 보호법 및 정보통신망 이용촉진 및 정보보호 등에 관한 법률 등 관련 법령을 준수합니다."
                </p>

                <h2 style=(h2_style())>"1. 수집하는 개인정보"</h2>
                <p style=(p_style())>"사이트는 서비스 이용을 위해 다음의 개인정보를 수집합니다."</p>
                <ul style=(list_style())>
                    <li><strong>"회원가입 시"</strong>": 이메일 주소, 사용자 이름, 표시 이름"</li>
                    <li><strong>"소셜 로그인 시"</strong>": GitHub/Google 프로필 정보(이메일, 사용자 이름, 프로필 이미지)"</li>
                    <li><strong>"자동 수집"</strong>": 쿠키, 서비스 이용 기록(IP 주소, 브라우저 정보)"</li>
                </ul>

                <h2 style=(h2_style())>"2. 개인정보의 수집 및 이용 목적"</h2>
                <ul style=(list_style())>
                    <li>"회원 식별 및 인증"</li>
                    <li>"서비스 제공 및 운영"</li>
                    <li>"커뮤니티 게시글 작성 및 관리"</li>
                    <li>"공지사항 전달"</li>
                    <li>"서비스 개선 및 신규 서비스 개발"</li>
                </ul>

                <h2 style=(h2_style())>"3. 개인정보의 보유 및 이용 기간"</h2>
                <p style=(p_style())>
                    "이용자의 개인정보는 원칙적으로 개인정보의 수집 및 이용 목적이 달성되면 지체 없이 파기합니다. 단, 관계 법령의 규정에 의하여 보존할 필요가 있는 경우 해당 법령에서 정한 기간 동안 보관합니다."
                </p>

                <h2 style=(h2_style())>"4. 개인정보의 제3자 제공"</h2>
                <p style=(p_style())>
                    "사이트는 이용자의 개인정보를 원칙적으로 외부에 제공하지 않습니다. 다만, 다음의 경우는 예외로 합니다:"
                </p>
                <ul style=(list_style())>
                    <li>"이용자가 사전에 동의한 경우"</li>
                    <li>"법령의 규정에 의거하거나, 수사 목적으로 법령에 정해진 절차와 방법에 따라 수사기관의 요구가 있는 경우"</li>
                </ul>

                <h2 style=(h2_style())>"5. 쿠키의 사용"</h2>
                <p style=(p_style())>
                    "사이트는 이용자 개개인의 접근성 및 편의성 증대를 위해 쿠키를 운용합니다. 쿠키는 웹 서버가 브라우저에 보내는 소량의 데이터이며, 이용자의 컴퓨터 하드디스크에 저장됩니다."
                </p>
                <p style=(p_style())>"사용 중인 쿠키:"</p>
                <ul style=(list_style())>
                    <li><strong>"__Host-session"</strong>": 로그인 세션 유지 (HttpOnly, SameSite=Lax, Secure, 서버사이드 검증)"</li>
                    <li><strong>"csrf_seed"</strong>", "<strong>"oauth_state"</strong>": 로그인·회원가입 폼 보호용 단기 쿠키 (약 10분)"</li>
                    <li><strong>"theme"</strong>": 다크모드/라이트모드 설정"</li>
                    <li><strong>"locale"</strong>": 언어 설정"</li>
                    <li><strong>"cookie_consent"</strong>": 쿠키 동의 여부"</li>
                </ul>
                <p style=(p_style())>
                    "이용자는 브라우저의 설정을 통해 쿠키 저장을 거부할 수 있습니다. 단, 쿠키 저장을 거부할 경우 서비스 이용에 제한이 있을 수 있습니다."
                </p>

                <h2 style=(h2_style())>"6. 광고 서비스"</h2>
                <p style=(p_style())>
                    "사이트는 Google AdSense를 통해 광고를 게재할 수 있습니다. Google은 쿠키를 사용하여 이전 방문 기반으로 광고를 제공합니다. Google의 쿠키 사용에 대한 자세한 내용은 "
                    <a href=(crate::util::external_links::GOOGLE_ADS_POLICIES) style="color:#2563EB;">"Google 광고 정책"</a>
                    "을 참조하십시오."
                </p>
                <p style=(p_style())>
                    "Google AdSense 광고가 활성화된 경우, 제3자 벤더인 Google이 쿠키와 웹 비콘을 사용하여 사이트 및 인터넷의 다른 사이트에 있는 사용자에게 광고를 게재합니다."
                </p>

                <h2 style=(h2_style())>"7. 개인정보 보호 책임자"</h2>
                <p style=(p_style())>
                    "개인정보 보호 관련 문의는 아래로 연락해 주시기 바랍니다."
                </p>
                <p style=(p_style())>(format!("이메일: {email}"))</p>

                <h2 style=(h2_style())>"8. 개인정보처리방침 변경"</h2>
                <p style=(p_style())>
                    "이 개인정보처리방침은 법령, 정책 또는 보안기술의 변경에 따라 내용이 추가, 삭제 및 수정될 수 있으며, 변경 시 사이트 공지사항을 통해 공지합니다."
                </p>
            }
        </div>
    })
}

#[page("/terms")]
pub async fn terms_of_service(cx: &Cx) -> Result<impl View + '_> {
    let loc = crate::services::i18n::locale(cx);
    let email = crate::config::contact_email();
    Ok(view! {
        <div style=(page_style())>
            if loc == Locale::En {
                <h1 style=(h1_style())>"Terms of Service"</h1>
                <p style="font-size:13px; color:#9ca3af; margin-bottom:16px;">"Last updated: October 9, 2026"</p>
                <p style=(note_style())>(EN_NOTICE)</p>

                <h2 style=(h2_style())>"Article 1 (Purpose)"</h2>
                <p style=(p_style())>
                    "These Terms set out the conditions and procedures for using all services provided by Rust Korea Hub (the \"Site\"), and the rights, obligations and responsibilities of users and the Site."
                </p>

                <h2 style=(h2_style())>"Article 2 (Definitions)"</h2>
                <ul style=(list_style())>
                    <li><strong>"\"User\""</strong>" means any member or non-member who accesses the Site and receives its services under these Terms."</li>
                    <li><strong>"\"Member\""</strong>" means a person who has registered with the Site and can continuously use its services."</li>
                    <li><strong>"\"Non-member\""</strong>" means a person who uses the Site's services without registering."</li>
                </ul>

                <h2 style=(h2_style())>"Article 3 (Effect and amendment of the Terms)"</h2>
                <p style=(p_style())>
                    "These Terms take effect when posted on the service or otherwise announced. The Site may amend these Terms when necessary, and amended Terms take effect upon announcement."
                </p>

                <h2 style=(h2_style())>"Article 4 (Services)"</h2>
                <p style=(p_style())>"The Site provides members with the following services:"</p>
                <ul style=(list_style())>
                    <li>"Rust manuals and learning materials"</li>
                    <li>"Community board (posts, comments, likes, bookmarks)"</li>
                    <li>"Curated tech videos"</li>
                    <li>"Event management"</li>
                    <li>"Other services determined by the Site"</li>
                </ul>
                <p style=(p_style())>
                    "The Site may change the content of its services and will announce such changes in advance."
                </p>

                <h2 style=(h2_style())>"Article 5 (Obligations of members)"</h2>
                <ul style=(list_style())>
                    <li>"Members must comply with applicable laws, these Terms, the usage guide and any notices related to the service."</li>
                    <li>"Members must not infringe the rights of others, including copyright and portrait rights."</li>
                    <li>"Members must not post inappropriate content such as spam, illegal advertising or hateful content."</li>
                </ul>

                <h2 style=(h2_style())>"Article 6 (Intellectual property)"</h2>
                <p style=(p_style())>
                    "Copyright in content created by the Site (manuals, UI, etc.) belongs to the Site. Copyright in posts written by users belongs to the users; by uploading a post, the user grants the Site a non-exclusive license to post, reproduce and display that content."
                </p>

                <h2 style=(h2_style())>"Article 7 (Management of posts)"</h2>
                <p style=(p_style())>"The Site may delete, without prior notice, posts or comments that:"</p>
                <ul style=(list_style())>
                    <li>"Defame or slander others"</li>
                    <li>"Are illegal or violate public order and good morals"</li>
                    <li>"Are spam, advertising or flooding"</li>
                    <li>"May infringe on personal privacy"</li>
                    <li>"Otherwise violate applicable laws"</li>
                </ul>

                <h2 style=(h2_style())>"Article 8 (Limitation of liability)"</h2>
                <p style=(p_style())>
                    "The Site is not liable for any damages related to services provided free of charge, nor for damages caused by a user's violation of these Terms."
                </p>

                <h2 style=(h2_style())>"Article 9 (Governing law and jurisdiction)"</h2>
                <p style=(p_style())>
                    "These Terms are governed by the laws of the Republic of Korea. Any dispute shall be subject to the jurisdiction of the Seoul Central District Court."
                </p>

                <h2 style=(h2_style())>"Article 10 (Contact)"</h2>
                <p style=(p_style())>"For questions about these Terms, please contact:"</p>
                <p style=(p_style())>(format!("Email: {email}"))</p>
            } else {
                <h1 style=(h1_style())>"이용약관"</h1>
                <p style="font-size:13px; color:#9ca3af; margin-bottom:32px;">"최종 수정일: 2026년 10월 9일"</p>

                <h2 style=(h2_style())>"제1조 (목적)"</h2>
                <p style=(p_style())>
                    "본 약관은 Rust Korea Hub(이하 '사이트')가 제공하는 모든 서비스의 이용 조건 및 절차, 이용자와 사이트 간의 권리, 의무 및 책임 사항을 규정함을 목적으로 합니다."
                </p>

                <h2 style=(h2_style())>"제2조 (정의)"</h2>
                <ul style=(list_style())>
                    <li><strong>"'이용자'"</strong>"란 사이트에 접속하여 이 약관에 따라 사이트가 제공하는 서비스를 받는 회원 및 비회원을 말합니다."</li>
                    <li><strong>"'회원'"</strong>"이란 사이트에 회원등록을 한 자로서, 지속적으로 사이트가 제공하는 서비스를 이용할 수 있는 자를 말합니다."</li>
                    <li><strong>"'비회원'"</strong>"이란 회원에 가입하지 않고 사이트가 제공하는 서비스를 이용하는 자를 말합니다."</li>
                </ul>

                <h2 style=(h2_style())>"제3조 (약관의 효력과 변경)"</h2>
                <p style=(p_style())>
                    "본 약관은 서비스 화면에 게시하거나 기타의 방법으로 공지함으로써 효력을 발생합니다. 사이트는 필요시 본 약관을 변경할 수 있으며, 변경된 약관은 공지와 동시에 그 효력을 발생합니다."
                </p>

                <h2 style=(h2_style())>"제4조 (서비스의 제공 및 변경)"</h2>
                <p style=(p_style())>
                    "사이트는 회원에게 아래와 같은 서비스를 제공합니다:"
                </p>
                <ul style=(list_style())>
                    <li>"Rust 관련 매뉴얼 및 학습 자료 제공"</li>
                    <li>"커뮤니티 게시판 (글 작성, 댓글, 좋아요, 북마크)"</li>
                    <li>"기술 영상 큐레이션"</li>
                    <li>"이벤트 관리"</li>
                    <li>"기타 사이트가 정하는 서비스"</li>
                </ul>
                <p style=(p_style())>
                    "사이트는 서비스의 내용을 변경할 수 있으며, 이 경우 변경 사항을 사전에 공지합니다."
                </p>

                <h2 style=(h2_style())>"제5조 (회원의 의무)"</h2>
                <ul style=(list_style())>
                    <li>"회원은 관계 법령, 본 약관의 규정, 이용안내 및 서비스와 관련하여 공지한 주의사항을 준수하여야 합니다."</li>
                    <li>"회원은 타인의 저작권, 초상권 등 제3자의 권리를 침해해서는 안 됩니다."</li>
                    <li>"회원은 스팸, 불법 광고, 혐오성 콘텐츠 등 부적절한 게시물을 작성해서는 안 됩니다."</li>
                </ul>

                <h2 style=(h2_style())>"제6조 (지적재산권)"</h2>
                <p style=(p_style())>
                    "사이트가 작성한 콘텐츠(매뉴얼, UI 등)의 저작권은 사이트에 있습니다. 이용자가 작성한 게시물의 저작권은 이용자에게 있으며, 이용자는 사이트에 게시물을 업로드함으로써 사이트에 해당 콘텐츠의 게시, 복제, 전시에 대한 비독점적 라이선스를 부여합니다."
                </p>

                <h2 style=(h2_style())>"제7조 (게시물의 관리)"</h2>
                <p style=(p_style())>
                    "사이트는 다음 각 호에 해당하는 게시물이나 댓글을 사전 통보 없이 삭제할 수 있습니다:"
                </p>
                <ul style=(list_style())>
                    <li>"타인을 비방하거나 명예를 훼손하는 내용"</li>
                    <li>"불법 콘텐츠 또는 공공질서 및 미풍양속에 위배되는 내용"</li>
                    <li>"스팸, 광고, 도배성 게시물"</li>
                    <li>"개인정보 침해 우려가 있는 내용"</li>
                    <li>"기타 관계 법령에 위배되는 내용"</li>
                </ul>

                <h2 style=(h2_style())>"제8조 (책임의 한계)"</h2>
                <p style=(p_style())>
                    "사이트는 무료로 제공되는 서비스와 관련하여 어떠한 손해도 책임지지 않습니다. 이용자가 본 약관을 위반하여 발생한 손해에 대해 사이트는 책임을 지지 않습니다."
                </p>

                <h2 style=(h2_style())>"제9조 (준거법 및 관할)"</h2>
                <p style=(p_style())>
                    "본 약관은 대한민국 법률에 따라 규율되며, 분쟁이 발생할 경우 서울중앙지방법원을 관할 법원으로 합니다."
                </p>

                <h2 style=(h2_style())>"제10조 (연락처)"</h2>
                <p style=(p_style())>
                    "본 약관에 대한 문의는 아래로 연락해 주시기 바랍니다."
                </p>
                <p style=(p_style())>(format!("이메일: {email}"))</p>
            }
        </div>
    })
}
