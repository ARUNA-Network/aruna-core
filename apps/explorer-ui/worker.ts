export default {
  async fetch(request: Request, env: any, ctx: any): Promise<Response> {
    // Dengan Cloudflare Workers + Assets, aset statis di dalam folder ./dist
    // akan dilayani secara otomatis oleh Cloudflare sebelum mencapai handler ini.
    //
    // Jika request sampai ke handler ini, berarti tidak ada aset statis yang cocok.
    // Kita bisa mengembalikan respon 404 atau melakukan penanganan kustom lainnya.
    return new Response("Asset not found", { status: 404 });
  },
};
