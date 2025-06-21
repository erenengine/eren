pub struct DirectionalLight {
    /// 빛의 방향 (단위 벡터, 월드 좌표 기준)
    pub direction: [f32; 3],

    /// 강도 (조도 조절용 스칼라값)
    pub intensity: f32,

    /// 빛의 색 (RGB, 보통 값은 0.0~1.0)
    pub color: [f32; 4],
}
