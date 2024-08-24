export default interface RelayState {
  id: string;
  nickname: string;
  circuits: Record<string, string>;
  streams: Record<string, string>;
  is_rendezvous_point: boolean;
  is_introduction_point: boolean;
  logs: string[];
}
