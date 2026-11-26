// Parcel's `worklet:` imports give back the URL of the compiled worklet file.
declare module "worklet:*" {
  const url: string;
  export default url;
}
