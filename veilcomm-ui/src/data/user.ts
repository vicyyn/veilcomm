export default interface UserState {
  id: string;
  nickname: string;
  introduction_points: Record<string, string>;
  rsa_public_key: number[];
  circuits: Record<string, string[]>;
  handshakes: Record<string, number[]>;
  connected_users: Record<string, number[]>;
  streams: Record<string, string>;
  logs: string[];
}