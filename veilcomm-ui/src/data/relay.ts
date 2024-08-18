export default interface RelayState {
  id: string;
  nickname: string;
  circuits: Record<string, string>;
  logs: string[];
}
